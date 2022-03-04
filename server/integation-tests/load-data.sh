#!/usr/bin/env bash

set -eux

http POST localhost:3030/_gadget/api/redirect 'X-Forwarded-User: 1234' 'X-Forwarded-Preferred-Username: Test User' alias=google destination="https://duckduckgo.com"
REDIRECT_ID="$(http GET localhost:3030/_gadget/api/redirect/google 'X-Forwarded-User: 1234' 'X-Forwarded-Preferred-Username: Test User' | jq -r '.data.public_ref')"
http PUT localhost:3030/_gadget/api/redirect/$REDIRECT_ID 'X-Forwarded-User: 1234' 'X-Forwarded-Preferred-Username: Test User' destination="https://yahoo.com"
http GET localhost:3030/_gadget/api/redirect 'X-Forwarded-User: 1234' 'X-Forwarded-Preferred-Username: Test User'

http GET localhost:3030/google