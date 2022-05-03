import { Stack, StackProps } from 'aws-cdk-lib';
import { HostedZone } from 'aws-cdk-lib/aws-route53';
import { Construct } from 'constructs';
import { Api } from './api';
import { Data } from './data';

export class AnalyticsPlatformStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const hosted_zone = HostedZone.fromLookup(this, 'HostedZoneFromLookup', {
      domainName: process.env.HOSTED_ZONE as string
    })

    const data = new Data(this, 'Data')
    new Api(this, 'Api', { table: data.table, hosted_zone: hosted_zone})

  }
}
