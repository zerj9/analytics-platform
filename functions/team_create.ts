import { Context, APIGatewayProxyResultV2, APIGatewayProxyEventV2WithLambdaAuthorizer } from 'aws-lambda';
import { DynamoDBClient, PutItemCommand } from "@aws-sdk/client-dynamodb";
import { LambdaAuthorizer } from './model';

const dynamodb = new DynamoDBClient({});

export const handler = async (event: APIGatewayProxyEventV2WithLambdaAuthorizer<LambdaAuthorizer>, _context: Context): Promise<APIGatewayProxyResultV2> => {
  const authorizer = event.requestContext.authorizer!.lambda;
  const userType = authorizer.user_type!

  const body = JSON.parse(event.body!);
  const teamName = body.team_name;

  console.log(`user_type: ${userType}`)

  if (userType == "SuperAdmin") {
    const command = new PutItemCommand({
      TableName: process.env.TABLE_NAME!,
      Item: {
        PK: { S: `TEAM#${teamName}` },
        SK: { S: `TEAM#${teamName}` }
      }
    })

    await dynamodb.send(command);

    return {
      statusCode: 200,
      body: ""
    }
  }

  return {
    statusCode: 403,
    body: ""
  }
};
