// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::{
    inetstack::{
        protocols::tcp::{
            tests::{
                check_packet_data,
                check_packet_pure_ack,
                setup::{
                    advance_clock,
                    connection_setup,
                },
            },
            SeqNumber,
        },
        test_helpers::{
            self,
            SharedEngine,
        },
    },
    runtime::{
        memory::DemiBuffer,
        network::consts::RECEIVE_BATCH_SIZE,
        Operation,
        OperationResult,
        OperationTask,
        QDesc,
    },
};
use ::anyhow::Result;
use ::futures::task::noop_waker_ref;
use ::rand;
use ::std::{
    collections::VecDeque,
    future::Future,
    net::SocketAddrV4,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    time::Instant,
};

//=============================================================================

/// Cooks a buffer.
fn cook_buffer(size: usize, stamp: Option<u8>) -> DemiBuffer {
    assert!(size < u16::MAX as usize);
    let mut buf: DemiBuffer = DemiBuffer::new(size as u16);
    for i in 0..size {
        buf[i] = stamp.unwrap_or(i as u8);
    }
    buf
}

//=============================================================================

fn send_data<const N: usize>(
    ctx: &mut Context,
    now: &mut Instant,
    receiver: &mut SharedEngine<N>,
    sender: &mut SharedEngine<N>,
    sender_qd: QDesc,
    window_size: u16,
    seq_no: SeqNumber,
    ack_num: Option<SeqNumber>,
    bytes: DemiBuffer,
) -> Result<(DemiBuffer, usize)> {
    trace!(
        "send_data ====> push: {:?} -> {:?}",
        sender.get_test_rig().get_ip_addr(),
        receiver.get_test_rig().get_ip_addr()
    );

    // Push data.
    let push_coroutine: Pin<Box<Operation>> = sender.tcp_push(sender_qd, bytes);
    let mut task: OperationTask = OperationTask::new(format!("test::send_data::push_coroutine"), push_coroutine);
    let bytes: DemiBuffer = sender.get_test_rig().pop_frame();
    let bufsize: usize = check_packet_data(
        bytes.clone(),
        sender.get_test_rig().get_link_addr(),
        receiver.get_test_rig().get_link_addr(),
        sender.get_test_rig().get_ip_addr(),
        receiver.get_test_rig().get_ip_addr(),
        window_size,
        seq_no,
        ack_num,
    )?;

    advance_clock(Some(receiver), Some(sender), now);

    // Push completes.
    match Future::poll(Pin::new(&mut task), ctx) {
        Poll::Ready(()) => match task.get_result().expect("coroutine should have completed") {
            (qd, OperationResult::Push) => {
                trace!("send_data ====> push completed");
                assert_eq!(sender_qd, qd);
                Ok((bytes, bufsize))
            },
            (_, _) => anyhow::bail!("coroutine should have returned push"),
        },
        _ => anyhow::bail!("push should have completed successfully"),
    }
}

//=============================================================================

fn recv_data<const N: usize>(
    ctx: &mut Context,
    receiver: &mut SharedEngine<N>,
    sender: &mut SharedEngine<N>,
    receiver_qd: QDesc,
    bytes: DemiBuffer,
) -> Result<()> {
    trace!(
        "recv_data ====> pop: {:?} -> {:?}",
        sender.get_test_rig().get_ip_addr(),
        receiver.get_test_rig().get_ip_addr()
    );

    // Pop data.
    let pop_coroutine: Pin<Box<Operation>> = receiver.tcp_pop(receiver_qd);
    let mut task: OperationTask = OperationTask::new(format!("test::recv_data::pop_coroutine"), pop_coroutine);

    if let Err(e) = receiver.receive(bytes) {
        anyhow::bail!("receive returned error: {:?}", e);
    }

    // Pop completes
    // Push completes.
    match Future::poll(Pin::new(&mut task), ctx) {
        Poll::Ready(()) => match task.get_result().expect("coroutine should have completed") {
            (qd, OperationResult::Pop(_, _)) => {
                trace!("send_data ====> push completed");
                assert_eq!(receiver_qd, qd);
                Ok(())
            },
            (_, _) => anyhow::bail!("coroutine should have returned pop"),
        },
        _ => anyhow::bail!("pop should have completed"),
    }
}

//=============================================================================

fn recv_pure_ack<const N: usize>(
    now: &mut Instant,
    sender: &mut SharedEngine<N>,
    receiver: &mut SharedEngine<N>,
    ack_num: SeqNumber,
) -> Result<()> {
    trace!(
        "recv_pure_ack ====> ack: {:?} -> {:?}",
        sender.get_test_rig().get_ip_addr(),
        receiver.get_test_rig().get_ip_addr()
    );

    advance_clock(Some(sender), Some(receiver), now);
    sender.get_test_rig().poll_scheduler();

    // Pop pure ACK
    if let Some(bytes) = sender.get_test_rig().pop_frame_unchecked() {
        check_packet_pure_ack(
            bytes.clone(),
            sender.get_test_rig().get_link_addr(),
            receiver.get_test_rig().get_link_addr(),
            sender.get_test_rig().get_ip_addr(),
            receiver.get_test_rig().get_ip_addr(),
            ack_num,
        )?;
        match receiver.receive(bytes) {
            Ok(()) => trace!("recv_pure_ack ====> ack completed"),
            Err(e) => anyhow::bail!("did not receive an ack: {:?}", e),
        }
    };

    Ok(())
}

//=============================================================================

fn send_recv<const N: usize>(
    ctx: &mut Context,
    now: &mut Instant,
    server: &mut SharedEngine<N>,
    client: &mut SharedEngine<N>,
    server_qd: QDesc,
    client_qd: QDesc,
    window_size: u16,
    seq_no: SeqNumber,
    bytes: DemiBuffer,
) -> Result<()> {
    let bufsize: usize = bytes.len();

    // Push data.
    let (bytes, _): (DemiBuffer, usize) = send_data(
        ctx,
        now,
        server,
        client,
        client_qd,
        window_size,
        seq_no,
        None,
        bytes.clone(),
    )?;

    // Pop data.
    recv_data(ctx, server, client, server_qd, bytes)?;

    // Pop pure ACK.
    recv_pure_ack(now, server, client, seq_no + SeqNumber::from(bufsize as u32))?;

    Ok(())
}

//=============================================================================

fn send_recv_round<const N: usize>(
    ctx: &mut Context,
    now: &mut Instant,
    server: &mut SharedEngine<N>,
    client: &mut SharedEngine<N>,
    server_qd: QDesc,
    client_qd: QDesc,
    window_size: u16,
    seq_no: SeqNumber,
    bytes: DemiBuffer,
) -> Result<()> {
    // Push Data: Client -> Server
    let (bytes, bufsize): (DemiBuffer, usize) =
        send_data(ctx, now, server, client, client_qd, window_size, seq_no, None, bytes)?;

    // Pop data.
    recv_data(ctx, server, client, server_qd, bytes.clone())?;

    // Push Data: Server -> Client
    let bytes: DemiBuffer = cook_buffer(bufsize, None);
    let (bytes, _): (DemiBuffer, usize) = send_data(
        ctx,
        now,
        client,
        server,
        server_qd,
        window_size,
        seq_no,
        Some(seq_no + SeqNumber::from(bufsize as u32)),
        bytes,
    )?;

    // Pop data.
    recv_data(ctx, client, server, client_qd, bytes.clone())?;

    Ok(())
}

//=============================================================================

fn connection_hangup<const N: usize>(
    _ctx: &mut Context,
    now: &mut Instant,
    server: &mut SharedEngine<N>,
    client: &mut SharedEngine<N>,
    server_qd: QDesc,
    client_qd: QDesc,
) -> Result<()> {
    // Send FIN: Client -> Server
    if let Err(e) = client.tcp_close(client_qd) {
        anyhow::bail!("client tcp_close returned error: {:?}", e);
    }
    client.get_test_rig().poll_scheduler();
    let bytes: DemiBuffer = client.get_test_rig().pop_frame();
    advance_clock(Some(server), Some(client), now);

    // ACK FIN: Server -> Client
    if let Err(e) = server.receive(bytes) {
        anyhow::bail!("server receive returned error: {:?}", e);
    }
    server.get_test_rig().poll_scheduler();
    let bytes: DemiBuffer = server.get_test_rig().pop_frame();
    advance_clock(Some(server), Some(client), now);

    // Receive ACK FIN
    if let Err(e) = client.receive(bytes) {
        anyhow::bail!("client receive (of ACK) returned error: {:?}", e);
    }
    advance_clock(Some(server), Some(client), now);

    // Send FIN: Server -> Client
    if let Err(e) = server.tcp_close(server_qd) {
        anyhow::bail!("server tcp_close returned error: {:?}", e);
    }
    server.get_test_rig().poll_scheduler();
    let bytes: DemiBuffer = server.get_test_rig().pop_frame();
    advance_clock(Some(server), Some(client), now);

    // ACK FIN: Client -> Server
    if let Err(e) = client.receive(bytes) {
        anyhow::bail!("client receive (of FIN) returned error {:?}", e);
    }
    client.get_test_rig().poll_scheduler();
    let bytes: DemiBuffer = client.get_test_rig().pop_frame();
    advance_clock(Some(server), Some(client), now);

    // Receive ACK FIN
    if let Err(e) = server.receive(bytes) {
        anyhow::bail!("server receive returned error: {:?}", e);
    }

    advance_clock(Some(server), Some(client), now);

    client.get_test_rig().poll_scheduler();
    server.get_test_rig().poll_scheduler();

    Ok(())
}

//=============================================================================

/// Tests one way communication. This should force the receiving peer to send
/// pure ACKs to the sender.
#[test]
pub fn test_send_recv_loop() -> Result<()> {
    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut now = Instant::now();

    // Connection parameters
    let listen_port: u16 = 80;
    let listen_addr: SocketAddrV4 = SocketAddrV4::new(test_helpers::BOB_IPV4, listen_port);

    // Setup peers.
    let mut server: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_bob2(now);
    let mut client: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_alice2(now);
    let window_scale: u8 = client.get_test_rig().get_tcp_config().get_window_scale();
    let max_window_size: u32 = match (client.get_test_rig().get_tcp_config().get_receive_window_size() as u32)
        .checked_shl(window_scale as u32)
    {
        Some(shift) => shift,
        None => anyhow::bail!("incorrect receive window"),
    };

    let ((server_qd, addr), client_qd): ((QDesc, SocketAddrV4), QDesc) =
        connection_setup(&mut now, &mut server, &mut client, listen_port, listen_addr)?;
    crate::ensure_eq!(addr.ip(), &test_helpers::ALICE_IPV4);

    let bufsize: u32 = 64;
    let buf: DemiBuffer = cook_buffer(bufsize as usize, None);

    for i in 0..((max_window_size + 1) / bufsize) {
        send_recv(
            &mut ctx,
            &mut now,
            &mut server,
            &mut client,
            server_qd,
            client_qd,
            max_window_size as u16,
            SeqNumber::from(1 + i * bufsize),
            buf.clone(),
        )?;
    }

    Ok(())
}

//=============================================================================

#[test]
pub fn test_send_recv_round_loop() -> Result<()> {
    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut now = Instant::now();

    // Connection parameters
    let listen_port: u16 = 80;
    let listen_addr: SocketAddrV4 = SocketAddrV4::new(test_helpers::BOB_IPV4, listen_port);

    // Setup peers.
    let mut server: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_bob2(now);
    let mut client: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_alice2(now);
    let window_scale: u8 = client.get_test_rig().get_tcp_config().get_window_scale();
    let max_window_size: u32 = match (client.get_test_rig().get_tcp_config().get_receive_window_size() as u32)
        .checked_shl(window_scale as u32)
    {
        Some(shift) => shift,
        None => anyhow::bail!("incorrect receive window"),
    };
    let ((server_qd, addr), client_qd): ((QDesc, SocketAddrV4), QDesc) =
        connection_setup(&mut now, &mut server, &mut client, listen_port, listen_addr)?;
    crate::ensure_eq!(addr.ip(), &test_helpers::ALICE_IPV4);

    let bufsize: u32 = 64;
    let buf: DemiBuffer = cook_buffer(bufsize as usize, None);

    for i in 0..((max_window_size + 1) / bufsize) {
        send_recv_round(
            &mut ctx,
            &mut now,
            &mut server,
            &mut client,
            server_qd,
            client_qd,
            max_window_size as u16,
            SeqNumber::from(1 + i * bufsize),
            buf.clone(),
        )?;
    }

    Ok(())
}

//=============================================================================

/// Tests one way communication, with some random transmission delay. This
/// should force the receiving peer to send pure ACKs to the sender, as well as
/// the sender side to trigger the RTO calculation logic.
#[test]
pub fn test_send_recv_with_delay() -> Result<()> {
    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut now = Instant::now();

    // Connection parameters
    let listen_port: u16 = 80;
    let listen_addr: SocketAddrV4 = SocketAddrV4::new(test_helpers::BOB_IPV4, listen_port);

    // Setup peers.
    let mut server: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_bob2(now);
    let mut client: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_alice2(now);
    let window_scale: u8 = client.get_test_rig().get_tcp_config().get_window_scale();
    let max_window_size: u32 = match (client.get_test_rig().get_tcp_config().get_receive_window_size() as u32)
        .checked_shl(window_scale as u32)
    {
        Some(shift) => shift,
        None => anyhow::bail!("incorrect receive window"),
    };

    let ((server_qd, addr), client_qd): ((QDesc, SocketAddrV4), QDesc) =
        connection_setup(&mut now, &mut server, &mut client, listen_port, listen_addr)?;
    crate::ensure_eq!(addr.ip(), &test_helpers::ALICE_IPV4);

    let bufsize: u32 = 64;
    let buf: DemiBuffer = cook_buffer(bufsize as usize, None);
    let mut recv_seq_no: SeqNumber = SeqNumber::from(1);
    let mut seq_no: SeqNumber = SeqNumber::from(1);
    let mut inflight = VecDeque::<DemiBuffer>::new();

    for _ in 0..((max_window_size + 1) / bufsize) {
        // Push data.
        let (bytes, _): (DemiBuffer, usize) = send_data(
            &mut ctx,
            &mut now,
            &mut server,
            &mut client,
            client_qd,
            max_window_size as u16,
            seq_no,
            None,
            buf.clone(),
        )?;

        seq_no = seq_no + SeqNumber::from(bufsize);

        inflight.push_back(bytes);

        // Pop data oftentimes.
        if rand::random() {
            if let Some(bytes) = inflight.pop_front() {
                recv_data(&mut ctx, &mut server, &mut client, server_qd, bytes.clone())?;
                recv_seq_no = recv_seq_no + SeqNumber::from(bufsize as u32);
            }
        }

        // Pop pure ACK
        recv_pure_ack(&mut now, &mut server, &mut client, recv_seq_no)?;
    }

    // Pop inflight packets.
    while let Some(bytes) = inflight.pop_front() {
        // Pop data.
        recv_data(&mut ctx, &mut server, &mut client, server_qd, bytes.clone())?;
        recv_seq_no = recv_seq_no + SeqNumber::from(bufsize as u32);

        // Recv pure ack (should also account for piggybacked ack).
        // FIXME: https://github.com/demikernel/demikernel/issues/680
        recv_pure_ack(&mut now, &mut server, &mut client, recv_seq_no)?;
    }

    Ok(())
}

//=============================================================================

#[test]
fn test_connect_disconnect() -> Result<()> {
    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut now = Instant::now();

    // Connection parameters
    let listen_port: u16 = 80;
    let listen_addr: SocketAddrV4 = SocketAddrV4::new(test_helpers::BOB_IPV4, listen_port);

    // Setup peers.
    let mut server: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_bob2(now);
    let mut client: SharedEngine<RECEIVE_BATCH_SIZE> = test_helpers::new_alice2(now);

    let ((server_qd, addr), client_qd): ((QDesc, SocketAddrV4), QDesc) =
        connection_setup(&mut now, &mut server, &mut client, listen_port, listen_addr)?;
    crate::ensure_eq!(addr.ip(), &test_helpers::ALICE_IPV4);

    connection_hangup(&mut ctx, &mut now, &mut server, &mut client, server_qd, client_qd)?;

    Ok(())
}
