# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

import subprocess
import time
from ci.task.windows import BaseWindowsTask, CheckoutOnWindows, CompileOnWindows, RunOnWindows, CleanupOnWindows
from ci.job.utils import wait_and_report

# ======================================================================================================================
# Remote Commands
# ======================================================================================================================


# Executes a test in a remote windows host.
def remote_run_windows(host: str, repository: str, is_debug: bool, target: str, is_sudo: bool, config_path: str):
    task: RunOnWindows = RunOnWindows(
        host, repository, target, is_debug, is_sudo, config_path)
    return task.execute()

# ======================================================================================================================
# Generic Jobs for Windows
# ======================================================================================================================


def job_checkout_windows(repository: str, branch: str, server: str, client: str, enable_nfs: bool,
                         log_directory: str) -> bool:
    test_name = "checkout"
    jobs: dict[str, subprocess.Popen[str]] = {}

    severTask: CheckoutOnWindows = CheckoutOnWindows(
        server, repository, branch)
    jobs[test_name + "-server-" + server] = severTask.execute()

    if not enable_nfs:
        clientTask: CheckoutOnWindows = CheckoutOnWindows(
            client, repository, branch)
        jobs[test_name + "-client-" + client] = clientTask.execute()

    return wait_and_report(test_name, log_directory, jobs)


def job_compile_windows(
        repository: str, libos: str, is_debug: bool, server: str, client: str, enable_nfs: bool,
        log_directory: str) -> bool:
    test_name = "compile-{}".format("debug" if is_debug else "release")
    jobs: dict[str, subprocess.Popen[str]] = {}

    serverTask: CompileOnWindows = CompileOnWindows(
        server, repository, "all LIBOS={}".format(libos), is_debug)
    jobs[test_name + "-server-" + server] = serverTask.execute()

    if not enable_nfs:
        clientTask: CompileOnWindows = CompileOnWindows(
            client, repository, "all LIBOS={}".format(libos), is_debug)
        jobs[test_name + "-client-" + client] = clientTask.execute()

    return wait_and_report(test_name, log_directory, jobs)


def job_test_system_rust_windows(
        test_alias: str, test_name: str, repo: str, libos: str, is_debug: bool, server: str, client: str,
        server_args: str, client_args: str, is_sudo: bool, all_pass: bool, delay: float, config_path: str,
        log_directory: str) -> bool:
    server_cmd: str = "test-system-rust LIBOS={} TEST={} ARGS='{}'".format(
        libos, test_name, server_args)
    client_cmd: str = "test-system-rust LIBOS={} TEST={} ARGS='{}'".format(
        libos, test_name, client_args)
    jobs: dict[str, subprocess.Popen[str]] = {}
    jobs[test_alias + "-server-" +
         server] = remote_run_windows(server, repo, is_debug, server_cmd, is_sudo, config_path)
    time.sleep(delay)
    jobs[test_alias + "-client-" +
         client] = remote_run_windows(client, repo, is_debug, client_cmd, is_sudo, config_path)
    return wait_and_report(test_alias, log_directory, jobs, all_pass)


def job_test_unit_rust_windows(repo: str, libos: str, is_debug: bool, server: str, client: str,
                               is_sudo: bool, config_path: str, log_directory: str) -> bool:
    server_cmd: str = "test-unit-rust LIBOS={}".format(libos)
    client_cmd: str = "test-unit-rust LIBOS={}".format(libos)
    test_name = "unit-test"
    jobs: dict[str, subprocess.Popen[str]] = {}
    jobs[test_name + "-server-" +
         server] = remote_run_windows(server, repo, is_debug, server_cmd, is_sudo, config_path)
    # Unit tests require a single endpoint, so do not run them on client.
    return wait_and_report(test_name, log_directory, jobs, True)


def job_test_integration_tcp_rust_windows(
        repo: str, libos: str, is_debug: bool, server: str, client: str, server_addr: str, client_addr: str,
        is_sudo: bool, config_path: str, log_directory: str) -> bool:
    server_args: str = "--local-address {}:12345 --remote-address {}:23456".format(
        server_addr, client_addr)
    client_args: str = "--local-address {}:23456 --remote-address {}:12345".format(
        client_addr, server_addr)
    server_cmd: str = "test-integration-rust TEST_INTEGRATION=tcp-test LIBOS={} ARGS='{}'".format(
        libos, server_args)
    client_cmd: str = "test-integration-rust TEST_INTEGRATION=tcp-test LIBOS={} ARGS='{}'".format(
        libos, client_args)
    test_name = "integration-test"
    jobs: dict[str, subprocess.Popen[str]] = {}
    jobs[test_name + "-server-" +
         server] = remote_run_windows(server, repo, is_debug, server_cmd, is_sudo, config_path)
    jobs[test_name + "-client-" + client] = remote_run_windows(
        client, repo, is_debug, client_cmd, is_sudo, config_path)
    return wait_and_report(test_name, log_directory, jobs, True)


def job_cleanup_windows(repository: str, server: str, client: str, is_sudo: bool, enable_nfs: bool, log_directory: str) -> bool:
    test_name = "cleanup"
    default_branch: str = "dev"
    jobs: dict[str, subprocess.Popen[str]] = {}

    serverTask: CleanupOnWindows = CleanupOnWindows(
        server, repository, is_sudo, default_branch)
    jobs[test_name + "-server-" + server] = serverTask.execute()

    if not enable_nfs:
        clientTask: CleanupOnWindows = CleanupOnWindows(
            client, repository, is_sudo, default_branch)
        jobs[test_name + "-client-" + client] = clientTask.execute()

    return wait_and_report(test_name, log_directory, jobs)
