import { APIGatewayRequestAuthorizerEventV2, APIGatewaySimpleAuthorizerWithContextResult, Context } from 'aws-lambda';
import { DynamoDBClient } from "@aws-sdk/client-dynamodb";
import { getSession, getUserFromId } from './model';

const dynamodb = new DynamoDBClient({});

export const handler = async (event: APIGatewayRequestAuthorizerEventV2, _context: Context): Promise<APIGatewaySimpleAuthorizerWithContextResult<Record<string, string>>> => {
  console.log(`Event: ${JSON.stringify(event, null, 2)}`);

  const sessionCookie = event.cookies.find(element => element.startsWith("session_id="))
  const sessionId = sessionCookie?.split("session_id=")[1]!

  console.log(`session id: ${sessionId}`);

  const session = await getSession(dynamodb, sessionId);
  console.log(`SESSION FOUND: ${session}`)

  const userRecord = await getUserFromId(dynamodb, session.userId);
  console.log(`USER FOUND: ${userRecord}`)

  return {
    isAuthorized: true,
    context: {
      "user_id": userRecord.id,
      "user_email": userRecord.email,
      "user_type": userRecord.type
    }
  };
};
