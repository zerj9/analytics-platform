import { HttpApi, HttpMethod } from '@aws-cdk/aws-apigatewayv2-alpha';
import { HttpLambdaIntegration } from '@aws-cdk/aws-apigatewayv2-integrations-alpha';
import { Architecture, Code, Function, Runtime } from "aws-cdk-lib/aws-lambda";
import { RetentionDays } from "aws-cdk-lib/aws-logs";
import { Construct } from "constructs";
import { ITable } from 'aws-cdk-lib/aws-dynamodb';

export interface ApiProps {
    table: ITable;
}

export class Api extends Construct {
    constructor(scope: Construct, id: string, props: ApiProps) {
        super(scope, id)

        const api = new HttpApi(this, 'AnalyticsPlatformApi')

        const authFunction = new Function(this, 'AuthFunction', {
            description: 'Auth endpoint for use by API Gateway',
            runtime: Runtime.PROVIDED_AL2,
            architecture: Architecture.ARM_64,
            handler: 'not.required',
            code: Code.fromAsset('functions/auth/target/lambda/auth/bootstrap.zip'),
            logRetention: RetentionDays.ONE_WEEK,
            environment: {
                'RUST_BACKTRACE': '1',
                'TABLE': props.table.tableName
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
