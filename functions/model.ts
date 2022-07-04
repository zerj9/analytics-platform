import { DynamoDBClient, GetItemCommand, PutItemCommand, QueryCommand } from "@aws-sdk/client-dynamodb";

export interface User {
  id: string;
  email: string;
  type: string;
  status: string;
}

export const getUserFromEmail = async (dynamodb: DynamoDBClient, email: string): Promise<User> => {
  const command = new QueryCommand({
    TableName: process.env.TABLE_NAME!,
    IndexName: "GSI1",
    KeyConditionExpression: "GSI1PK = :email and GSI1SK = :email",
    ExpressionAttributeValues: {":email": {"S": `EMAIL#${email}`}, ":U": {"S": "USER#"}},
    FilterExpression: "begins_with(PK, :U) and begins_with(SK, :U)"
  })

  const dbResponse = await dynamodb.send(command);

  if (dbResponse.Count != 1) {
    throw Error(`User record for ${email} not found`)
  }

  const item = dbResponse.Items![0]

  return {
    id: item.PK.S!.split("USER#")[1],
    email: item.GSI1PK.S!.split("EMAIL#")[1],
    type: item.user_type.S!,
    status: item.status.S!
  }
}

export const getUserFromId = async (dynamodb: DynamoDBClient, userId: string): Promise<User> => {
  const command = new GetItemCommand({
    TableName: process.env.TABLE_NAME!,
    Key: {
      PK: { S: `USER#${userId}` },
      SK: { S: `USER#${userId}` }
    }
  })

  const dbResponse = await dynamodb.send(command);
  const item = dbResponse.Item!

  return {
    id: item.PK.S!.split("USER#")[1],
    email: item.GSI1PK.S!.split("EMAIL#")[1],
    type: item.user_type.S!,
    status: item.status.S!
  }
}

export interface AuthSession {
  id: string,
  userId: string,
  code: string,
  expiry: Date
}

export const getAuthSessions = async (dynamodb: DynamoDBClient, user: User): Promise<AuthSession[]> => {
  const command = new QueryCommand({
    TableName: process.env.TABLE_NAME!,
    IndexName: "GSI1",
    KeyConditionExpression: "GSI1PK = :user_id and GSI1SK = :user_id",
    ExpressionAttributeValues: {
      ":user_id": { S:`USER#${user.id}` },
      ":U": { S: "USER#" },
      ":AS": { S: "AUTHSESSION#" }
    },
    FilterExpression: "begins_with(PK, :U) and begins_with(SK, :AS)"
  })

  const dbResponse = await dynamodb.send(command);
  if (dbResponse.Count == 0) throw Error(`No auth sessions found for ${user.id}`);

  const authSessions: AuthSession[] = [];
  
  for (const record of dbResponse.Items!) {
    const authSession: AuthSession = {
      userId: record.PK.S!.split("USER#")[1],
      id: record.SK.S!.split("AUTHSESSION#")[1],
      code: record.code.S!,
      expiry: new Date(Number(record.expiry.N!) * 1000)
    }
    authSessions.push(authSession)
  }

  return authSessions
}

export interface Session {
  id: string,
  userId: string,
  csrfToken: string,
  expiry: Date
}

export const getSession = async (dynamodb: DynamoDBClient, sessionId: string): Promise<Session> => {
  const command = new QueryCommand({
    TableName: process.env.TABLE_NAME!,
    IndexName: "GSI1",
    KeyConditionExpression: "GSI1PK = :session_id and GSI1SK = :session_id",
    ExpressionAttributeValues: {
      ":session_id": { S:`SESSION#${sessionId}` },
      ":U": { S: "USER#" },
      ":AS": { S: "SESSION#" }
    },
    FilterExpression: "begins_with(PK, :U) and begins_with(SK, :AS)"
  });

  const dbResponse = await dynamodb.send(command);
  if (dbResponse.Count != 1) throw Error(`Session not found for ${sessionId}`);
  const sessionRecord = dbResponse.Items![0]

  return {
    id: sessionRecord.SK.S!.split("SESSION#")[1],
    userId: sessionRecord.PK.S!.split("USER#")[1],
    csrfToken: sessionRecord.csrf_token.S!,
    expiry: new Date(Number(sessionRecord.expiry.N!) * 1000)
  }
}

export interface LambdaAuthorizer {
  user_type: string
  user_email: string
  user_id: string
  }

export interface Team {
  name: string,
  admins: string[],
  users: string[]
}

export enum MemberType {
  Admin = "Admin",
  Maintain = "Maintain",
  User = "User"
}

export const getTeam = async (dynamodb: DynamoDBClient, teamName: string): Promise<Team> => {
  const command = new GetItemCommand({
    TableName: process.env.TABLE_NAME!,
    Key: {
      PK: { S: `TEAM#${teamName}` },
      SK: { S: `TEAM#${teamName}` }
    }
  })

  const dbResponse = await dynamodb.send(command);
  const item = dbResponse.Item!

  return {
    name: item.PK.S!,
    admins: [],
    users: []
  }
}

export enum ToolType {
  Jupyter = "Jupyter",
  RStudio = "RStudio",
  VSCode = "VSCode"
}

export interface Tool {
  id: string,
  userId: string,
  type: ToolType,
  version: string,
  cpu: string,
  memory: string
}

export const createTool = async (dynamodb: DynamoDBClient, tool: Tool): Promise<void> => {
  const command = new PutItemCommand({
    TableName: process.env.TABLE_NAME!,
    Item: {
      PK: { S: `USER#${tool.userId}` },
      SK: { S: `TOOL#${tool.id}` },
      GSI1PK: { S: `TOOLTYPE#${tool.type}` },
      GSI1SK: { S: `TOOLVERSION#${tool.version}` },
      cpu: { S: tool.cpu },
      memory: { S: tool.memory },
      status: { S: "creating" }
    }
  })
  await dynamodb.send(command);
}
