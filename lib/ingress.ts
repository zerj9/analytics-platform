import { Construct } from 'constructs';
import { Certificate, CertificateValidation } from 'aws-cdk-lib/aws-certificatemanager';
import { Port, SubnetType } from 'aws-cdk-lib/aws-ec2';
import { ICluster, ContainerImage, FargateService, ListenerConfig, LogDrivers, TaskDefinition, Compatibility, NetworkMode, ContainerDependencyCondition} from 'aws-cdk-lib/aws-ecs';
import { INetworkLoadBalancer, Protocol } from 'aws-cdk-lib/aws-elasticloadbalancingv2';
import { ARecord, IHostedZone, RecordTarget } from 'aws-cdk-lib/aws-route53';
import { LoadBalancerTarget } from 'aws-cdk-lib/aws-route53-targets';
import { BlockPublicAccess, Bucket, BucketEncryption } from 'aws-cdk-lib/aws-s3';
import { BucketDeployment, Source } from 'aws-cdk-lib/aws-s3-deployment';
import { Role, ServicePrincipal } from 'aws-cdk-lib/aws-iam';
import * as path from 'path';

export interface IngressProps {
  readonly ingressCluster: ICluster
  readonly nlb: INetworkLoadBalancer
  readonly hostedZone: IHostedZone
}

export class Ingress extends Construct {
  constructor(scope: Construct, id: string, props: IngressProps) {
    super(scope, id);

    const toolsUrl = `*.tools.${props.hostedZone.zoneName}`
    const volumeName = "envoyconfig"

    const configBucket = new Bucket(this, 'ConfigBucket', {
      blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
      encryption: BucketEncryption.S3_MANAGED,
      enforceSSL: true,
      versioned: true,
    })

    new BucketDeployment(this, 'EnvoyYamlToS3', {
      sources: [Source.asset(path.join(__dirname, ".."), {exclude: ["**", ".**", "!envoy.yaml", "!cds.json", "!lds.json", "!envoy_pull_xds_config.sh"]})],
      destinationBucket: configBucket,
      prune: false
    })

    const envoyConfigTaskRole = new Role(this, 'EnvoyConfigTaskRole', {
      assumedBy: new ServicePrincipal('ecs-tasks.amazonaws.com')
    })

    configBucket.grantRead(envoyConfigTaskRole)

    const certificate = new Certificate(this, 'ToolsCertificate', {
      domainName: toolsUrl,
      validation: CertificateValidation.fromDns(props.hostedZone)
    })

    const envoyTaskDefinition = new TaskDefinition(this, 'EnvoyTaskDefinition', {
      compatibility: Compatibility.FARGATE,
      cpu: "512",
      memoryMiB: "1024",
      networkMode: NetworkMode.AWS_VPC,
      taskRole: envoyConfigTaskRole
    })
    envoyTaskDefinition.addVolume({name: volumeName, host: {}})

    const initialConfigContainer = envoyTaskDefinition.addContainer('InitialConfigSync', {
      image: ContainerImage.fromRegistry("amazon/aws-cli:2.7.1"),
      cpu: 256,
      memoryLimitMiB: 512,
      containerName: "InitialConfigSync",
      essential: false,
      logging: LogDrivers.awsLogs({ streamPrefix: 'Envoy' }),
      command: ["s3", "sync", `s3://${configBucket.bucketName}`, "/etc/envoy"]
    })

    initialConfigContainer.addMountPoints({
      containerPath: "/etc/envoy",
      readOnly: false,
      sourceVolume: volumeName
    })

    const xdsSyncContainer = envoyTaskDefinition.addContainer('XDSConfigSync', {
      image: ContainerImage.fromRegistry("amazon/aws-cli:2.7.1"),
      cpu: 256,
      memoryLimitMiB: 512,
      containerName: "XDSConfigSync",
      essential: true,
      logging: LogDrivers.awsLogs({ streamPrefix: 'Envoy' }),
      entryPoint: ["/bin/bash"],
      command: ["/etc/envoy/envoy_pull_xds_config.sh", `s3://${configBucket.bucketName}`, "/etc/envoy"]
    })

    xdsSyncContainer.addContainerDependencies({ container: initialConfigContainer, condition: ContainerDependencyCondition.SUCCESS })
    
    xdsSyncContainer.addMountPoints({
      containerPath: "/etc/envoy",
      readOnly: false,
      sourceVolume: volumeName
    })

    const envoyContainer = envoyTaskDefinition.addContainer('EnvoyContainer', {
      image: ContainerImage.fromRegistry("envoyproxy/envoy:v1.22-latest"),
      containerName: "envoy",
      essential: true,
      portMappings: [{containerPort: 8443}, { containerPort: 8080 }],
      logging: LogDrivers.awsLogs({ streamPrefix: 'Envoy' })
    })
    
    envoyContainer.addMountPoints({
      containerPath: "/etc/envoy",
      readOnly: true,
      sourceVolume: volumeName
    })

    envoyContainer.addContainerDependencies({ container: initialConfigContainer, condition: ContainerDependencyCondition.SUCCESS })

    const envoyFargateService = new FargateService(this, 'EnvoyService', {
      cluster: props.ingressCluster,
      desiredCount: 1,
      taskDefinition: envoyTaskDefinition,
      enableExecuteCommand: true,
      assignPublicIp: false,
      vpcSubnets: {
        subnetType: SubnetType.PRIVATE_WITH_NAT
      }

    })
    envoyFargateService.connections.allowFromAnyIpv4(Port.tcp(8443))

    const listener = props.nlb.addListener('EnvoyListener', {
      certificates: [certificate],
      port: 443,
      protocol: Protocol.TLS
    });

    envoyFargateService.registerLoadBalancerTargets({
      containerName: "envoy",
      newTargetGroupId: 'toolsListener',
      listener: ListenerConfig.networkListener(listener, {
        protocol: Protocol.TCP,
        port: 8443
      }),
      containerPort: 8443
    })

    new ARecord(this, 'EnvoyRoute53Record', {
      zone: props.hostedZone,
      recordName: toolsUrl,
      target: RecordTarget.fromAlias(new LoadBalancerTarget(props.nlb)
      )
    })
  }
}
