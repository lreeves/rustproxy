#!/bin/bash

set -e
set -x

curl -i -s -x http://localhost:3128 http://www.cnn.com/ 1>/dev/null

echo "Complete"
