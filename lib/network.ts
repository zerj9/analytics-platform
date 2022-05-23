import { Construct } from 'constructs';
import { SubnetType, Vpc } from "aws-cdk-lib/aws-ec2";
import { Cluster } from 'aws-cdk-lib/aws-ecs';
import { NetworkLoadBalancer } from 'aws-cdk-lib/aws-elasticloadbalancingv2';

export class Network extends Construct {
  public readonly vpc: Vpc;
  public readonly ingressCluster: Cluster;
  public readonly nlb: NetworkLoadBalancer;

  constructor(scope: Construct, id: string) {
    super(scope, id);

    this.vpc = new Vpc(this, 'Vpc', {
      cidr: "10.0.0.0/16",
      enableDnsHostnames: false,
      natGateways: 1, // Increase for HA
      subnetConfiguration: [
        {
          name: 'public',
          subnetType: SubnetType.PUBLIC,
          cidrMask: 24,
        },
        {
          name: 'private',
          subnetType: SubnetType.PRIVATE_WITH_NAT,
          cidrMask: 20,
        }
      ]
    })

    this.ingressCluster = new Cluster(this, 'IngressCluster', {
      vpc: this.vpc
    })

    this.nlb = new NetworkLoadBalancer(this, 'nlb', {
      vpc: this.vpc,
      crossZoneEnabled: true,
      internetFacing: true,
      vpcSubnets: {
        subnets: this.vpc.publicSubnets
      }
    })
  }
}
