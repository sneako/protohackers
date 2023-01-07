{:ok, socket} = :gen_tcp.connect('localhost', 7878, mode: :binary, active: false)

:gen_tcp.send(socket, <<73, 0, 0, 48, 57, 0, 0, 0, 101>>)
:gen_tcp.send(socket, <<73, 0, 0, 48, 58, 0, 0, 0, 102>>)
:gen_tcp.send(socket, <<73, 0, 0, 48, 59, 0, 0, 0, 103>>)
# query
:gen_tcp.send(socket, <<51, 0, 0, 0, 0, 1, 0, 0, 103>>)

:gen_tcp.recv(socket, 9)
|> dbg()
