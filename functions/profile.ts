import { Context, APIGatewayProxyCallback, APIGatewayEvent } from 'aws-lambda';

export const handler = (event: APIGatewayEvent, _context: Context, callback: APIGatewayProxyCallback): void => {
    const authorizer = event.requestContext.authorizer!.lambda;

    callback(null, {
        statusCode: 200,
        body: JSON.stringify({
            email: authorizer.user_email,
            user_id: authorizer.user_id,
        }),
    });
};
