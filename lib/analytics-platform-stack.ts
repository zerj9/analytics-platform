import { Stack, StackProps } from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { Data } from './data';

export class AnalyticsPlatformStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    new Data(this, 'Data')
  }
}
