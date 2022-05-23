import { Stack, StackProps } from 'aws-cdk-lib';
import { HostedZone } from 'aws-cdk-lib/aws-route53';
import { Construct } from 'constructs';
import { Api } from './api';
import { Data } from './data';
import { Ingress } from './ingress';
import { Network } from './network';

export class AnalyticsPlatformStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const hostedZone = HostedZone.fromLookup(this, 'HostedZoneFromLookup', {
      domainName: process.env.HOSTED_ZONE as string
    })

    const network = new Network(this, 'Network')
    const ingress = new Ingress(this, 'Ingress', {
      ingressCluster: network.ingressCluster,
      nlb: network.nlb,
      hostedZone: hostedZone
     });
    const data = new Data(this, 'Data')
    new Api(this, 'Api', { table: data.table, hostedZone: hostedZone})

  }
}
