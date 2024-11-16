// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

// This should come first.
// Glibc macro to expose definitions corresponding to the POSIX.1-2008 base specification.
// See https://man7.org/linux/man-pages/man7/feature_test_macros.7.html.
#define _POSIX_C_SOURCE 200809L
#define _GNU_SOURCE
/*====================================================================================================================*
 * Imports                                                                                                            *
 *====================================================================================================================*/

#include <assert.h>
#include <demi/libos.h>
#include <demi/sga.h>
#include <demi/wait.h>
#include <signal.h>
#include <string.h>
#include <pthread.h>
#include <numa.h>
#include <sched.h>

#ifdef __linux__
#include <arpa/inet.h>
#include <sys/socket.h>
#endif

#include <time.h>

#include "common.h"

/*====================================================================================================================*
 * Constants                                                                                                          *
 *====================================================================================================================*/

/**
 * @brief Data size.
 */
#define DATA_SIZE 64

/**
 * @brief Maximum number of iterations.
 */
#define MAX_ITERATIONS 1000000

/*====================================================================================================================*
 * push_and_wait()                                                                                                    *
 *====================================================================================================================*/

/**
 * @brief Pushes a scatter-gather array to a remote socket and waits for operation to complete.
 *
 * @param qd   Target queue descriptor.
 * @param sga  Target scatter-gather array.
 * @param qr   Storage location for operation result.
 * @param dest Destination address.
 */
static void pushto_wait(int qd, demi_sgarray_t *sga, demi_qresult_t *qr, const struct sockaddr *dest)
{
    demi_qtoken_t qt = -1;

    /* Push data. */
    assert(demi_pushto(&qt, qd, sga, (const struct sockaddr *)dest, sizeof(struct sockaddr_in)) == 0);

    /* Wait push operation to complete. */
    assert(demi_wait(qr, qt, NULL) == 0);

    /* Parse operation result. */
    assert(qr->qr_opcode == DEMI_OPC_PUSH);
}

/*====================================================================================================================*
 * pop_and_wait()                                                                                                    *
 *====================================================================================================================*/

/**
 * @brief Pops a scatter-gather array and waits for operation to complete.
 *
 * @param qd Target queue descriptor.
 * @param qr Storage location for operation result.
 */
static void pop_wait(int qd, demi_qresult_t *qr)
{
    demi_qtoken_t qt = -1;

    /* Pop data. */
    assert(demi_pop(&qt, qd) == 0);

    /* Wait for pop operation to complete. */
    assert(demi_wait(qr, qt, NULL) == 0);

    /* Parse operation result. */
    assert(qr->qr_opcode == DEMI_OPC_POP);
    assert(qr->qr_value.sga.sga_segs != 0);
}

static pthread_mutex_t demi_init_lock = PTHREAD_MUTEX_INITIALIZER;
static int demi_initialized = 0;

void init_demi_kernel(struct demi_args *args) {
    // printf("Initializing Demikernel\n");
    pthread_mutex_lock(&demi_init_lock);
    // if (!demi_initialized) {
        assert(demi_init(args) == 0);
        demi_initialized = 1;
        // printf("Demikernel initialized globally.\n");
    // }
    pthread_mutex_unlock(&demi_init_lock);
}

typedef struct {
    struct demi_args *args;
    struct sockaddr_in *local;
    struct sockaddr_in *remote;
    size_t data_size;
    unsigned max_iterations;
} thread_args_t;

/*====================================================================================================================*
 * server()                                                                                                           *
 *====================================================================================================================*/

/**
 * @brief UDP echo server.
 *
 * @param argc   Argument count.
 * @param argv   Argument list.
 * @param local  Local socket address.
 * @param remote Remote socket address.
 * @param max_iterations Maximum number of iterations.
 */
static void server(thread_args_t *targs)
{
    int sockqd = -1;
    init_demi_kernel(targs->args);

    /* Setup local socket. */
    assert(demi_socket(&sockqd, AF_INET, SOCK_DGRAM, 0) == 0);
    assert(demi_bind(sockqd, (const struct sockaddr *)targs->local, sizeof(struct sockaddr_in)) == 0);

    /* Run. */
    for (unsigned it = 0; it < targs->max_iterations; it++)
    {
        demi_qresult_t qr = {0};
        demi_sgarray_t sga = {0};

        /* Pop scatter-gather array. */
        pop_wait(sockqd, &qr);

        /* Extract received scatter-gather array. */
        memcpy(&sga, &qr.qr_value.sga, sizeof(demi_sgarray_t));

        /* Push scatter-gather array. */
        pushto_wait(sockqd, &sga, &qr, (const struct sockaddr *)targs->remote);

        /* Release received scatter-gather array. */
        assert(demi_sgafree(&sga) == 0);

        // fprintf(stdout, "ping (%u)\n", it);
    }
}

void *thread_server(void *args) {
    server((thread_args_t *)args);
    return NULL;
}

/*====================================================================================================================*
 * client()                                                                                                           *
 *====================================================================================================================*/

/**
 * @brief UDP echo client.
 *
 * @param argc   Argument count.
 * @param argv   Argument list.
 * @param local  Local socket address.
 * @param remote Remote socket address.
 * @param data_size Number of bytes in each message.
 * @param max_iterations Maximum number of iterations.
 */
static void client(thread_args_t *targs)
{

#ifdef _WIN32
    char expected_buf[DATA_SIZE];
#endif

#ifdef __linux__
    char expected_buf[targs->data_size];
#endif

    int sockqd = -1;
    init_demi_kernel(targs->args);

    /* Setup local socket. */
    assert(demi_socket(&sockqd, AF_INET, SOCK_DGRAM, 0) == 0);
    assert(demi_bind(sockqd, (const struct sockaddr *)targs->local, sizeof(struct sockaddr_in)) == 0);

    /* Run. */
    printf("Running client\n");

    for (unsigned it = 0; it < targs->max_iterations; it++)
    {
        demi_qresult_t qr = {0};
        demi_sgarray_t sga = {0};

        /* Allocate scatter-gather array. */
        sga = demi_sgaalloc(targs->data_size);
        assert(sga.sga_segs != 0);
        // printf("Allocated scatter-gather array, len: %u, addr: %p\n", sga.sga_segs[0].sgaseg_len, sga.sga_segs[0].sgaseg_buf);

        /* Prepare data. */
        memset(expected_buf, it % 256, targs->data_size);
        // printf("set %lu data\n", data_size);
        memcpy(sga.sga_segs[0].sgaseg_buf, expected_buf, targs->data_size);
        // printf("Prepared data\n");

        /* Push scatter-gather array. */
        pushto_wait(sockqd, &sga, &qr, (const struct sockaddr *)targs->remote);
        // printf("Pushed scatter-gather array\n");

        /* Release sent scatter-gather array. */
        assert(demi_sgafree(&sga) == 0);
        // printf("Freed scatter-gather array\n");

        /* Pop data scatter-gather array. */
        pop_wait(sockqd, &qr);

        /* Parse operation result. */
        assert(!memcmp(qr.qr_value.sga.sga_segs[0].sgaseg_buf, expected_buf, targs->data_size));

        /* Release received scatter-gather array. */
        assert(demi_sgafree(&qr.qr_value.sga) == 0);

        // fprintf(stdout, "pong (%u)\n", it);
    }
}

void *thread_client(void *args) {
    client((thread_args_t *)args);
    return NULL;
}

/*====================================================================================================================*
 * usage()                                                                                                            *
 *====================================================================================================================*/

/**
 * @brief Prints program usage and exits.
 *
 * @param progname Program name.
 */
static void usage(const char *progname)
{
    fprintf(stderr, "Usage: %s MODE local-ipv4 local-port remote-ipv4 remote-port\n", progname);
    fprintf(stderr, "Modes:\n");
    fprintf(stderr, "  --client    Run program in client mode.\n");
    fprintf(stderr, "  --server    Run program in server mode.\n");

    exit(EXIT_SUCCESS);
}

/*====================================================================================================================*
 * build_sockaddr()                                                                                                   *
 *====================================================================================================================*/

/**
 * @brief Builds a socket address.
 *
 * @param ip_str    String representation of an IP address.
 * @param port_str  String representation of a port number.
 * @param addr      Storage location for socket address.
 */
void build_sockaddr(const char *const ip_str, const char *const port_str, struct sockaddr_in *const addr)
{
    int port = -1;

    sscanf(port_str, "%d", &port);
    addr->sin_family = AF_INET;
    addr->sin_port = htons(port);
    assert(inet_pton(AF_INET, ip_str, &addr->sin_addr) == 1);
}

/*====================================================================================================================*
 * main()                                                                                                             *
 *====================================================================================================================*/

/**
 * @brief Exercises a one-way direction communication through UDP.
 *
 * This system-level test instantiates two demikernel nodes: a client and a server. The client sends UDP packets to the
 * server in a tight loop. The server process in a tight loop received UDP packets from the client.
 *
 * @param argc Argument count.
 * @param argv Argument list.
 *
 * @return On successful completion EXIT_SUCCESS is returned.
 */
int main(int argc, char *const argv[])
{
    if (argc >= 6)
    {
        reg_sighandlers();

        // Parameters
        size_t data_size = DATA_SIZE;
        unsigned max_iterations = MAX_ITERATIONS;
        unsigned num_threads = 1;
        int default_local_port = atoi(argv[3]);
        int default_remote_port = atoi(argv[5]);
        if (argc >= 7)
            sscanf(argv[6], "%zu", &data_size);
        if (argc >= 8)
            sscanf(argv[7], "%u", &max_iterations);
        if (argc >= 9)
            sscanf(argv[8], "%u", &num_threads);
        printf("Data size: %zu\n", data_size);
        printf("Max iterations: %u\n", max_iterations);
        printf("Num threads: %u\n", num_threads);
        
        // Init thread args
        thread_args_t thread_args[num_threads];
        pthread_t threads[num_threads];
        struct demi_args args = {
            .argc = argc,
            .argv = argv,
            .callback = NULL,
        };

        struct sockaddr_in *local = (struct sockaddr_in *)malloc(num_threads * sizeof(struct sockaddr_in));
        struct sockaddr_in *remote = (struct sockaddr_in *)malloc(num_threads * sizeof(struct sockaddr_in));
        char *local_port_str = (char *)malloc(6 * sizeof(char));
        char *remote_port_str = (char *)malloc(6 * sizeof(char));

        for (u_int16_t i = 0; i < num_threads; i++) {
            sprintf(local_port_str, "%d", default_local_port + i);
            build_sockaddr(argv[2], local_port_str, &local[i]);
            thread_args[i].local = &local[i];
            sprintf(remote_port_str, "%d", default_remote_port + i);
            build_sockaddr(argv[4], remote_port_str, &remote[i]);
            thread_args[i].remote = &remote[i];
            thread_args[i].data_size = data_size;
            thread_args[i].max_iterations = max_iterations;
            thread_args[i].args = &args;
        }
        printf("Finished building sockaddr\n");

        // set affinity
        int cpu_ids[num_threads];
        for (u_int16_t i = 0; i < num_threads; i++) {
            if (i < 8)
                cpu_ids[i] = i + 8;
            else
                cpu_ids[i] = i + 16;
        }

        if (!strcmp(argv[1], "--server"))
        {
            for (u_int16_t i = 0; i < num_threads; i++) {
                if (pthread_create(&threads[i], NULL, thread_server, &thread_args[i]) != 0) {
                    fprintf(stderr, "Error creating thread\n");
                    return 1;
                }
                // server(thread_args[i].sockqd, thread_args[i].remote, thread_args[i].max_iterations);
            }
        }
        else if (!strcmp(argv[1], "--client"))
        {
            for (u_int16_t i = 0; i < num_threads; i++) {
                if (pthread_create(&threads[i], NULL, thread_client, &thread_args[i]) != 0) {
                    fprintf(stderr, "Error creating thread\n");
                    return 1;
                }
                // client(&thread_args[i]);
            }
        }
        // bind to core
        for (u_int16_t i = 0; i < num_threads; i++) {
            cpu_set_t cpuset;
            CPU_ZERO(&cpuset);
            CPU_SET(cpu_ids[i], &cpuset);
            pthread_setaffinity_np(threads[i], sizeof(cpu_set_t), &cpuset);
        }
        clock_t start, end;
        double cpu_time_used;
        start = clock();
        // Wait for threads to finish
        for (u_int16_t i = 0; i < num_threads; i++) {
            pthread_join(threads[i], NULL);
        }
        end = clock();
        cpu_time_used = ((double) (end - start)) / CLOCKS_PER_SEC;
        printf("Time taken: %f\n", cpu_time_used);
        return (EXIT_SUCCESS);
    }

    usage(argv[0]);

    /* Never gets here. */

    return (EXIT_SUCCESS);
}
