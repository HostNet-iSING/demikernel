# Set location for Demikernel's config file.
export CONFIG_PATH=/home/ubuntu/git_repos/demikernel/scripts/config/config.yaml
export LD_LIBRARY_PATH=/home/ubuntu/git_repos/PipeTune/third_party/dpdk-stable-22.11.3/build/install/lib/x86_64-linux-gnu

# Set parameters for Demikernel's TCP/UDP stack.
export MSS=1500
export MTU=1500
export SERVER_IPV4_ADDR=10.0.2.102:56789
export CLIENT_IPV4_ADDR=10.0.2.101:56789

# Run system-level tests with Catnap LibOS, unless otherwise stated.
# If you want to system-level tests on a different LibOS, then set the LIBOS flag accordingly.
export LIBOS=catnip

SHELL_DIR=$(dirname "$0")
PROJ_DIR=$SHELL_DIR/..

# Run tcp-push-pop.
# $PROJ_DIR/bin/examples/rust/tcp-push-pop.elf --server $SERVER_IPV4_ADDR # Run this on server host.
# $PROJ_DIR/bin/examples/rust/tcp-push-pop.elf --client $SERVER_IPV4_ADDR # Run this on client host.

# Run tcp-ping-pong.
# $PROJ_DIR/bin/examples/rust/tcp-ping-pong.elf --server $SERVER_IPV4_ADDR # Run this on server host.
# $PROJ_DIR/bin/examples/rust/tcp-ping-pong.elf --client $SERVER_IPV4_ADDR # Run this on client host.

# Run udp-push-pop.
# $PROJ_DIR/bin/examples/rust/udp-push-pop.elf --server $SERVER_IPV4_ADDR $CLIENT_IPV4_ADDR # Run this on server host.
# $PROJ_DIR/bin/examples/rust/udp-push-pop.elf --client $CLIENT_IPV4_ADDR $SERVER_IPV4_ADDR # Run this on client host.

# Run udp-ping-pong.
# $PROJ_DIR/bin/examples/rust/udp-ping-pong.elf --server $SERVER_IPV4_ADDR $CLIENT_IPV4_ADDR # Run this on server host.
$PROJ_DIR/bin/examples/rust/udp-ping-pong.elf --client $CLIENT_IPV4_ADDR $SERVER_IPV4_ADDR # Run this on client host.
