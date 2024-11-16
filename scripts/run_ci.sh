# Set parameters for remote machines.
export SERVER_HOSTNAME=Desktop_02
export SERVER_IPV4_ADDR=192.168.40.171
export CLIENT_HOSTNAME=Desktop_01
export CLIENT_IPV4_ADDR=192.168.40.167
export DEMIKERNEL_PATH='/home/ubuntu/git_repos/demikernel'
export DEMIKERNEL_BRANCH=dev

# Run system-level tests with Catnap LibOS, unless otherwise stated.
# If you want to run system-level tests on a different LibOS, then set the LIBOS flag accordingly.
export LIBOS=catnap

# Run all system-level tests for the target LIBOS.
python3 tools/demikernel_ci.py \
    --platform linux \
    --server $SERVER_HOSTNAME \
    --client $CLIENT_HOSTNAME \
    --repository $DEMIKERNEL_PATH \
    --branch $DEMIKERNEL_BRANCH \
    --libos $LIBOS \
    --test-system tcp_echo \
    --server-addr $SERVER_IPV4_ADDR \
    --client-addr $CLIENT_IPV4_ADDR