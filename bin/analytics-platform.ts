#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { AnalyticsPlatformStack } from '../lib/analytics-platform-stack';

const app = new cdk.App();
new AnalyticsPlatformStack(app, 'AnalyticsPlatformStack', {
  env: { account: process.env.CDK_DEFAULT_ACCOUNT, region: process.env.CDK_DEFAULT_REGION },
});
