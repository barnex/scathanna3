//!
//!  Functionality for selecting a server and joining a game.
//!

use super::internal::*;

// Show "Select Server" menu if there are multiple options.
//pub(crate) fn select_server_menu(win: &mut Shell, servers: &[String]) -> Result<String> {
//	match servers {
//		[] => Err(anyhow!("No servers available in settings.toml")),
//		[server] => Ok(server.clone()),
//		list => Ok(list[ui::menu(win, "Select server:", &list)?].clone()),
//	}
//}

// connect to server
pub(crate) fn connect(server: &str, join_req: JoinRequest) -> Result<(NetPipe<ClientMsg, ServerMsg>, AcceptedMsg)> {
	LOG.write(format!("Connecting to {server}..."));
	let mut tcp_stream = TcpStream::connect(&server)?;
	LOG.write(format!("Connected. Joining..."));
	wireformat::serialize_into(&mut tcp_stream, &join_req)?;
	let accepted_msg: AcceptedMsg = wireformat::deserialize_from(&mut tcp_stream) //
		.map_err(|e| anyhow!("reading accept message: {e}"))?;
	let player_id = accepted_msg.player_id;
	LOG.write(format!("Accepted as player {player_id}"));
	let conn = NetPipe::new(tcp_stream);
	Ok((conn, accepted_msg))
}
