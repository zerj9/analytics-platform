import { Context, APIGatewayProxyCallback, APIGatewayEvent, APIGatewayProxyResult } from 'aws-lambda';
import { DeleteItemCommand, DynamoDBClient, PutItemCommand } from "@aws-sdk/client-dynamodb";
import { AttributeValue } from '@aws-sdk/client-dynamodb';
import { serialize } from 'cookie';
import { v4 as uuidv4 } from 'uuid';
import { AuthSession, getAuthSessions, getUserFromEmail, Session, User } from './model';

const dynamodb = new DynamoDBClient({});

export const handler = async (event: APIGatewayEvent, _context: Context, callback: APIGatewayProxyCallback): Promise<void> => {
    const query_params = event.queryStringParameters!;

    if (query_params.type && query_params.type == "login" && query_params.email) {
        callback(null, await login(dynamodb, query_params.email))

    } else if (query_params.type && query_params.type == "authenticate" && query_params.email && query_params.code) {
        callback(null, await authenticate(dynamodb, query_params.email, query_params.code))
    }
};


const login = async (dynamodb: DynamoDBClient, email: string): Promise<APIGatewayProxyResult> => {
    const user = await getUserFromEmail(dynamodb, email);

    if (user.status != 'active') {
        return {
            statusCode: 401,
            body: "unauthorized"
        }
    }
    await createSession(dynamodb, user);

    return {
        statusCode: 200,
        body: ""
    }
}

const authenticate = async (dynamodb: DynamoDBClient, email: string, code: string): Promise<APIGatewayProxyResult> => {
    const user = await getUserFromEmail(dynamodb, email);
    const authSessions = await getAuthSessions(dynamodb, user);
    const authSession = authSessions.find(sess => sess.code == code);
    if (authSession == undefined) return { statusCode: 401, body: "" }

    console.log(`session matched. id: ${authSession.id}`);
    const session = await createSession(dynamodb, user, false);
    deleteSession(dynamodb, authSession);

    const sessionCookie = serialize('session_id', session.id, {
        domain: process.env.HOSTED_ZONE!,
        secure: true,
        httpOnly: true,
        expires: session.expiry
    });
    
    const csrfCookie = serialize('csrf_token', session.csrfToken, {
        domain: process.env.HOSTED_ZONE!,
        secure: true,
        expires: session.expiry
    });

    return {
        statusCode: 200,
        body: "",
        headers: { // Multi Value Headers not supported by payload format 2.0
            "Set-Cookie": sessionCookie,
            "set-cookie": csrfCookie
        }
    }
}


const createSession = async (dynamodb: DynamoDBClient, user: User, authSession: boolean = true): Promise<Session> => {
    const sessionId = uuidv4();
    const csrfToken = uuidv4();
    const code = Array.from(Array(8), () => Math.floor(Math.random() * 36).toString(36)).join('').toUpperCase();

    const item: Record<string, AttributeValue> = {
        PK: { S: `USER#${user.id}` },
    }

    const dateNowUtc = new Date(new Date().toUTCString())
    let expiry: Date;

    if (authSession) {
        item.SK = { S: `AUTHSESSION#${sessionId}` }
        item.GSI1PK = { S: `USER#${user.id}` },
        item.GSI1SK = { S: `USER#${user.id}` },
        item.code = { S: code }
        expiry = new Date(dateNowUtc.getTime() + 300000) // 5 Minutes
        item.expiry = { N: String(expiry.getTime()/1000) }
    } else {
        item.SK = { S: `SESSION#${sessionId}` }
        item.GSI1PK = { S: `SESSION#${sessionId}` },
        item.GSI1SK = { S: `SESSION#${sessionId}` },
        expiry = new Date(dateNowUtc.getTime() + 28800000) // 8 Hours
        item.expiry = { N: String(expiry.getTime()/1000) }
        item.csrf_token = { S: csrfToken }
    }

    const command = new PutItemCommand({
        TableName: process.env.TABLE_NAME!,
        Item: item
    })

    await dynamodb.send(command);

    return {
        id: sessionId,
        userId: user.id,
        csrfToken: csrfToken,
        expiry: expiry
    }
}

const deleteSession = async (dynamodb: DynamoDBClient, session: AuthSession) => {
    const command = new DeleteItemCommand({
        TableName: process.env.TABLE_NAME!,
        Key: { PK: { S: `USER#${session.userId}` }, SK: { S: `AUTHSESSION#${session.id}` }}
    })

    await dynamodb.send(command);
}
