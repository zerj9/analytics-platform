import * as cdk from 'aws-cdk-lib';
import { Match, Template } from 'aws-cdk-lib/assertions';
import { Api } from '../lib/api';
import { Data } from '../lib/data';

test('Api Created', () => {
    // WHEN
    const app = new cdk.App();
    const stack = new cdk.Stack(app, 'MyTestStack');
    const data = new Data(stack, 'Data')
    new Api(stack, 'Api', { table: data.table });
    // THEN
    const template = Template.fromStack(stack);

    // Table has Partition/Hash key named PK and Sort/Range key named SK
    template.hasResourceProperties('AWS::DynamoDB::Table', {
        KeySchema: [
            {
                "AttributeName": "PK",
                "KeyType": "HASH"
            },
            {
                "AttributeName": "SK",
                "KeyType": "RANGE"
            }
        ]
    });

    // Two GSIs have been created
    template.hasResourceProperties('AWS::DynamoDB::Table', {
        GlobalSecondaryIndexes: Match.arrayWith([
            Match.objectLike({ "IndexName": "GSI1" }),
            Match.objectLike({ "IndexName": "GSI2" })
        ])
    });
});
