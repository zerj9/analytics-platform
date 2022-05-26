# Analytics Platform

## Deployment

### Diagram:

- To do

### Steps:

- To do

## Data Model

| Entity       | PK          | SK               | GSI1PK        | GSI1SK        | GSI2PK | GSI2SK |
| ------------ | ----------- | ---------------- | ------------- | ------------- | ------ | ------ |
| User         | USER#{id}   | USER#{id}        | EMAIL#{email} | EMAIL#{email} |        |        |
| Auth Session | USER#{id}   | AUTHSESSION#{id} | USER#{id}     | USER#{id}     |        |        |
| Session      | USER#{id}   | SESSION#{id}     | SESSION#{id}  | SESSION#{id}  |        |        |
| Team         | TEAM#{name} | TEAM#{name}      |               |               |        |        |
| Team Member  | TEAM#{name} | USER#{id}        | USER#{id}     | TEAM#{name}   |        |        |

## Extra attributes:

### User:

- first_name
- last_name
- user_type: [SuperAdmin, User]

### Auth Session:

- code: generated code used to authenticate (passwordless)
- expiry: timestamp (DynamoDB TTL)

### Session:

- csrf_token
- expiry: timestamp (DynamoDB TTL)
