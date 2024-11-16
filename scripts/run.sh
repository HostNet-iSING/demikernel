#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

REMOTE_HOST=$1
RUNCMD=$2

# SSH Options
SSH_OPTIONS="-p 30041"

#===============================================================================

set -e

ssh $SSH_OPTIONS $REMOTE_HOST "bash -l -c 'sudo -E $RUNCMD 2>&1 | tee \$HOSTNAME'"
