import { ApiMapping, DomainName, HttpApi, HttpMethod } from '@aws-cdk/aws-apigatewayv2-alpha';
import { HttpLambdaAuthorizer, HttpLambdaResponseType } from '@aws-cdk/aws-apigatewayv2-authorizers-alpha';
import { HttpLambdaIntegration } from '@aws-cdk/aws-apigatewayv2-integrations-alpha';
import { Duration } from 'aws-cdk-lib';
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
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
        const authFunction = new Function(this, 'AuthFunction', {
            description: 'Auth endpoint for use by API Gateway',
            runtime: Runtime.PROVIDED_AL2,
            architecture: Architecture.ARM_64,
            handler: 'not.required',
            code: Code.fromAsset('functions/target/lambda/auth/bootstrap.zip'),
            logRetention: RetentionDays.ONE_WEEK,
            environment: {
                'RUST_BACKTRACE': '1',
                'TABLE': props.table.tableName,
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

        // AUTHORIZER
        const lambdaAuthorizerFunction = new Function(this, 'LambdaAuthorizerFunction', {
            description: 'Lambda Authorizer function',
            runtime: Runtime.PROVIDED_AL2,
            architecture: Architecture.ARM_64,
            handler: 'not.required',
            code: Code.fromAsset('functions/target/lambda/authorizer/bootstrap.zip'),
            logRetention: RetentionDays.ONE_WEEK,
            environment: {
                'RUST_BACKTRACE': '1',
                'TABLE': props.table.tableName,
            }
        })
        props.table.grantReadWriteData(lambdaAuthorizerFunction);
        const authorizer = new HttpLambdaAuthorizer('LambdaAuthorizer', lambdaAuthorizerFunction, {
            responseTypes: [HttpLambdaResponseType.SIMPLE],
            identitySource: [],
            resultsCacheTtl: Duration.seconds(0)
        });
        // AUTHORIZER

        // PROFILE
        const profileFunction = new Function(this, 'ProfileFunction', {
            description: 'Profile endpoint for use by API Gateway',
            runtime: Runtime.PROVIDED_AL2,
            architecture: Architecture.ARM_64,
            handler: 'not.required',
            code: Code.fromAsset('functions/target/lambda/profile/bootstrap.zip'),
            logRetention: RetentionDays.ONE_WEEK,
            environment: {
                'RUST_BACKTRACE': '1',
                'TABLE': props.table.tableName,
            }
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
        const createTeamFunction = new Function(this, 'CreateTeamFunction', {
            description: 'Team POST endpoint for use by API Gateway',
            runtime: Runtime.PROVIDED_AL2,
            architecture: Architecture.ARM_64,
            handler: 'not.required',
            code: Code.fromAsset('functions/target/lambda/team_create/bootstrap.zip'),
            logRetention: RetentionDays.ONE_WEEK,
            environment: {
                'RUST_BACKTRACE': '1',
                'TABLE': props.table.tableName,
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

        // ADD TEAM USER
        const addTeamUserFunction = new Function(this, 'addTeamUserFunction', {
            description: 'Team POST endpoint to add user for use by API Gateway',
            runtime: Runtime.PROVIDED_AL2,
            architecture: Architecture.ARM_64,
            handler: 'not.required',
            code: Code.fromAsset('functions/target/lambda/team_add/bootstrap.zip'),
            logRetention: RetentionDays.ONE_WEEK,
            environment: {
                'RUST_BACKTRACE': '1',
                'TABLE': props.table.tableName,
            }
        })
        props.table.grantReadWriteData(addTeamUserFunction);
        api.addRoutes({
            path: '/team/{team_name}',
            methods: [HttpMethod.POST],
            integration: new HttpLambdaIntegration('AddTeamUserIntegration', addTeamUserFunction),
            authorizer: authorizer
        })
        // ADD TEAM USER
    }
}
