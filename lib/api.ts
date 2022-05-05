import { ApiMapping, DomainName, HttpApi, HttpMethod } from '@aws-cdk/aws-apigatewayv2-alpha';
import { HttpLambdaIntegration } from '@aws-cdk/aws-apigatewayv2-integrations-alpha';
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { Construct } from "constructs";
import { Certificate, CertificateValidation } from 'aws-cdk-lib/aws-certificatemanager';
import { ITable } from 'aws-cdk-lib/aws-dynamodb';
import { ARecord, IHostedZone, RecordTarget } from 'aws-cdk-lib/aws-route53';
import { ApiGatewayv2DomainProperties } from 'aws-cdk-lib/aws-route53-targets';

export interface ApiProps {
    table: ITable;
    hosted_zone: IHostedZone
}

export class Api extends Construct {
    constructor(scope: Construct, id: string, props: ApiProps) {
        super(scope, id)

        const apiUrl = `api.${props.hosted_zone.zoneName}`

        const api = new HttpApi(this, 'AnalyticsPlatformApi')
        const certificate = new Certificate(this, 'Certificate', {
            domainName: apiUrl,
            validation: CertificateValidation.fromDns(props.hosted_zone)
        })

        const domainName = new DomainName(this, 'ApiDomainName', {
            domainName: apiUrl,
            certificate: certificate,
        })

        new ARecord(this, 'CustomDomainAliasRecord', {
            zone: props.hosted_zone,
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
                'HOSTED_ZONE': props.hosted_zone.zoneName
            }
        })
        props.table.grantReadWriteData(authFunction);
        api.addRoutes({
            path: '/auth',
            methods: [HttpMethod.POST],
            integration: new HttpLambdaIntegration('AuthIntegration', authFunction)
        })
    }
}
