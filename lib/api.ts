import { ApiMapping, DomainName, HttpApi, HttpMethod } from '@aws-cdk/aws-apigatewayv2-alpha';
import { HttpLambdaAuthorizer, HttpLambdaResponseType } from '@aws-cdk/aws-apigatewayv2-authorizers-alpha';
import { HttpLambdaIntegration } from '@aws-cdk/aws-apigatewayv2-integrations-alpha';
import { Duration } from 'aws-cdk-lib';
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { NodejsFunction } from 'aws-cdk-lib/aws-lambda-nodejs';
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { Construct } from "constructs";
import { Certificate, CertificateValidation } from 'aws-cdk-lib/aws-certificatemanager';
import { ITable } from 'aws-cdk-lib/aws-dynamodb';
import { ARecord, IHostedZone, RecordTarget } from 'aws-cdk-lib/aws-route53';
import { ApiGatewayv2DomainProperties } from 'aws-cdk-lib/aws-route53-targets';

export interface ApiProps {
  table: ITable;
  hostedZone: IHostedZone
}

export class Api extends Construct {
  constructor(scope: Construct, id: string, props: ApiProps) {
    super(scope, id)

    const apiUrl = `api.${props.hostedZone.zoneName}`

    const api = new HttpApi(this, 'AnalyticsPlatformApi')
    const certificate = new Certificate(this, 'Certificate', {
      domainName: apiUrl,
      validation: CertificateValidation.fromDns(props.hostedZone)
    })

    const domainName = new DomainName(this, 'ApiDomainName', {
      domainName: apiUrl,
      certificate: certificate,
    })

    new ARecord(this, 'CustomDomainAliasRecord', {
      zone: props.hostedZone,
      recordName: apiUrl,
      target: RecordTarget.fromAlias(new ApiGatewayv2DomainProperties(
        domainName.regionalDomainName,
        domainName.regionalHostedZoneId)
      )
    });

    new ApiMapping(this, 'ApiMapping', {
      api: api,
      domainName: domainName
    })

    // AUTH
    const authFunction = new NodejsFunction(this, 'AuthFunction', {
      runtime: Runtime.NODEJS_16_X,
      architecture: Architecture.ARM_64,
      entry: "functions/auth.ts",
      logRetention: RetentionDays.ONE_WEEK,
      environment: {
      'TABLE_NAME': props.table.tableName,
      'HOSTED_ZONE': props.hostedZone.zoneName
      }
    })
    props.table.grantReadWriteData(authFunction);
    api.addRoutes({
      path: '/auth',
      methods: [HttpMethod.POST],
      integration: new HttpLambdaIntegration('AuthIntegration', authFunction)
    })
    // AUTH

    // LAMBDA AUTHORIZER
    const lambdaAuthorizerFunction = new NodejsFunction(this, 'LambdaAuthorizerFunction', {
      runtime: Runtime.NODEJS_16_X,
      architecture: Architecture.ARM_64,
      entry: "functions/authorizer.ts",
      logRetention: RetentionDays.ONE_WEEK,
      environment: {
        'TABLE_NAME': props.table.tableName,
      }
    })
    props.table.grantReadWriteData(lambdaAuthorizerFunction);
    const authorizer = new HttpLambdaAuthorizer('LambdaAuthorizer', lambdaAuthorizerFunction, {
      responseTypes: [HttpLambdaResponseType.SIMPLE],
      identitySource: [],
      resultsCacheTtl: Duration.seconds(0)
    });
    // LAMBDA AUTHORIZER

    // PROFILE
    const profileFunction = new NodejsFunction(this, 'ProfileFunction', {
      runtime: Runtime.NODEJS_16_X,
      architecture: Architecture.ARM_64,
      entry: "functions/profile.ts",
    })
    props.table.grantReadWriteData(profileFunction);
    api.addRoutes({
      path: '/profile',
      methods: [HttpMethod.GET],
      integration: new HttpLambdaIntegration('ProfileIntegration', profileFunction),
      authorizer: authorizer
    })
    // PROFILE

    // CREATE TEAM
    const createTeamFunction = new NodejsFunction(this, 'CreateTeamFunction', {
      runtime: Runtime.NODEJS_16_X,
      architecture: Architecture.ARM_64,
      entry: "functions/team_create.ts",
      environment: {
        'TABLE_NAME': props.table.tableName,
      }
    })
    props.table.grantReadWriteData(createTeamFunction);
    api.addRoutes({
      path: '/team',
      methods: [HttpMethod.POST],
      integration: new HttpLambdaIntegration('CreateTeamIntegration', createTeamFunction),
      authorizer: authorizer
    })
    // CREATE TEAM

    // ADD TEAM MEMBER
    const addTeamMemberFunction = new NodejsFunction(this, 'AddTeamMemberFunction', {
      runtime: Runtime.NODEJS_16_X,
      architecture: Architecture.ARM_64,
      entry: "functions/team_add.ts",
    })
    props.table.grantReadWriteData(addTeamMemberFunction);
    api.addRoutes({
      path: '/team/{team_name}',
      methods: [HttpMethod.POST],
      integration: new HttpLambdaIntegration('AddTeamMemberIntegration', addTeamMemberFunction),
      authorizer: authorizer
    })
    // ADD TEAM MEMBER

    // CREATE TOOL
    const toolCreateFunction = new NodejsFunction(this, 'ToolCreateFunction', {
      runtime: Runtime.NODEJS_16_X,
      architecture: Architecture.ARM_64,
      entry: "functions/tool_create.ts",
      environment: {
        'TABLE_NAME': props.table.tableName,
      }
    })
    props.table.grantReadWriteData(toolCreateFunction);
    api.addRoutes({
      path: '/tool',
      methods: [HttpMethod.POST],
      integration: new HttpLambdaIntegration('ToolCreateIntegration', toolCreateFunction),
      authorizer: authorizer
    })
    // CREATE TOOL
  }
}
