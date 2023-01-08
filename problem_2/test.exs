address = '137.66.59.170'
# address = 'localhost'
{:ok, socket} = :gen_tcp.connect(address, 7878, mode: :binary, active: false)

# query
:gen_tcp.send(socket, <<81, 0, 0, 0, 0, 1, 0, 0, 103>>)

{:ok, <<0, 0, 0, 0>>} = :gen_tcp.recv(socket, 0)

# insert
:gen_tcp.send(socket, <<73, 0, 0, 48, 57, 0, 0, 0, 101>>)
:gen_tcp.send(socket, <<73, 0, 0, 48, 58, 0, 0, 0, 102>>)
:gen_tcp.send(socket, <<73, 0, 0, 48, 59, 0, 0, 0, 103>>)

# query
:gen_tcp.send(socket, <<81, 0, 0, 0, 0, 1, 0, 0, 103>>)

{:ok, <<0, 0, 0, 102>>} = :gen_tcp.recv(socket, 0)

for i <- 59..65, do: :gen_tcp.send(socket, <<73, 0, 0, 48, i, 0, 0, 0, 210>>)

# query
:gen_tcp.send(socket, <<81, 0, 0, 0, 0, 1, 0, 0, 103>>)

{:ok, <<0, 0, 0, 185>>} = :gen_tcp.recv(socket, 0)

# invalid message
:gen_tcp.send(socket, <<1, 0, 0, 48, 0, 0, 0, 64, 0>>)

# query
:gen_tcp.send(socket, <<81, 0, 0, 0, 0, 1, 0, 0, 103>>)

{:ok, <<0, 0, 0, 185>>} = :gen_tcp.recv(socket, 0)
# {:error, :closed} = :gen_tcp.recv(socket, 0)
