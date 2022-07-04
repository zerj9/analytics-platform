import { Context, APIGatewayProxyEventV2WithLambdaAuthorizer, APIGatewayProxyResultV2 } from 'aws-lambda';
import { DynamoDBClient, PutItemCommand } from "@aws-sdk/client-dynamodb";
import { getTeam, getUserFromId, LambdaAuthorizer, MemberType, Team, User } from './model';

const dynamodb = new DynamoDBClient({});

export const handler = async (event: APIGatewayProxyEventV2WithLambdaAuthorizer<LambdaAuthorizer>, _context: Context): Promise<APIGatewayProxyResultV2> => {
  if (event.requestContext.authorizer.lambda.user_type != "SuperAdmin") {
    return { statusCode: 403, body: "" }
  }

  const queryParams = event.queryStringParameters!;
  const teamNameParam = queryParams.team_name!
  const team = await getTeam(dynamodb, teamNameParam);

  const body = JSON.parse(event.body!);
  const userId = body.user_id as string
  const user = await getUserFromId(dynamodb, userId);
  const memberType = body.member_type as MemberType

  await addUserToTeam(dynamodb, user, team, memberType)
  return { statusCode: 200, body: "" }  
};

const addUserToTeam = async (dynamodb: DynamoDBClient, user: User, team: Team, memberType: MemberType) => {
  const command = new PutItemCommand({
    TableName: process.env.TABLE_NAME!,
    Item: {
      PK: { S: `TEAM#${team.name}` },
      SK: { S: `USER#${user.id}` },
      GSI1PK: { S: `USER#${user.id}` },
      GSI1SK: { S: `TEAM#${team.name}` },
      member_type: { S: memberType }
    }
  })
  await dynamodb.send(command);
}
