# Set location for Demikernel's config file.
export CONFIG_PATH=/home/hxy/git_repos/demikernel/scripts/config/config.yaml

# Set parameters for Demikernel's TCP/UDP stack.
export MSS=1500
export MTU=1500
export SERVER_IPV4_ADDR=192.168.1.246:56789
export CLIENT_IPV4_ADDR=192.168.1.243:56789

# Run system-level tests with Catnap LibOS, unless otherwise stated.
# If you want to system-level tests on a different LibOS, then set the LIBOS flag accordingly.
export LIBOS=catnip

CUR_DIR=$PWD/..

# Run tcp-push-pop.
# $CUR_DIR/bin/examples/rust/tcp-push-pop.elf --server $SERVER_IPV4_ADDR # Run this on server host.
# $CUR_DIR/bin/examples/rust/tcp-push-pop.elf --client $SERVER_IPV4_ADDR # Run this on client host.

# Run tcp-ping-pong.
# $CUR_DIR/bin/examples/rust/tcp-ping-pong.elf --server $SERVER_IPV4_ADDR # Run this on server host.
# $CUR_DIR/bin/examples/rust/tcp-ping-pong.elf --client $SERVER_IPV4_ADDR # Run this on client host.

# Run udp-push-pop.
# $CUR_DIR/bin/examples/rust/udp-push-pop.elf --server $SERVER_IPV4_ADDR $CLIENT_IPV4_ADDR # Run this on server host.
# $CUR_DIR/bin/examples/rust/udp-push-pop.elf --client $CLIENT_IPV4_ADDR $SERVER_IPV4_ADDR # Run this on client host.

# Run udp-ping-pong.
# $CUR_DIR/bin/examples/rust/udp-ping-pong.elf --server $SERVER_IPV4_ADDR $CLIENT_IPV4_ADDR # Run this on server host.
$CUR_DIR/bin/examples/rust/udp-ping-pong.elf --client $CLIENT_IPV4_ADDR $SERVER_IPV4_ADDR # Run this on client host.
