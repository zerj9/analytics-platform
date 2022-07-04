import { Context, APIGatewayProxyEventV2WithLambdaAuthorizer, APIGatewayProxyResultV2 } from 'aws-lambda';
import { LambdaAuthorizer } from './model';

export const handler = async (event: APIGatewayProxyEventV2WithLambdaAuthorizer<LambdaAuthorizer>, _context: Context): Promise<APIGatewayProxyResultV2> => {
  const authorizer = event.requestContext.authorizer!.lambda;
    
  return {
    statusCode: 200,
    body: JSON.stringify({
      email: authorizer.user_email,
      user_id: authorizer.user_id,
      user_type: authorizer.user_type
    })
  }
};
