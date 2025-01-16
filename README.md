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
9. Build docker container:
```
docker build -t HTTPShellContainer
```
