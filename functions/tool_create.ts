import { Context, APIGatewayProxyResultV2, APIGatewayProxyEventV2WithLambdaAuthorizer } from 'aws-lambda';
import { DynamoDBClient, PutItemCommand } from "@aws-sdk/client-dynamodb";
import { LambdaAuthorizer, Tool, createTool } from './model';

const dynamodb = new DynamoDBClient({});

export const handler = async (event: APIGatewayProxyEventV2WithLambdaAuthorizer<LambdaAuthorizer>, _context: Context): Promise<APIGatewayProxyResultV2> => {
  const authorizer = event.requestContext.authorizer!.lambda;
  const body = JSON.parse(event.body!);

  const toolId = Array.from(Array(8), () => Math.floor(Math.random() * 36).toString(36)).join('').toUpperCase();
  const tool: Tool = {
    id: toolId,
    userId: authorizer.user_id,
    type: body.tool_type,
    version: body.tool_version,
    cpu: body.cpu,
    memory: body.memory
  }

  await createTool(dynamodb, tool);

  return {
    statusCode: 200,
    body: "config pass"
  }
};
