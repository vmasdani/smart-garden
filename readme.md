# Smart Garden  

A full-stack local web-app in Orange Pi Zero and Rust to manage home garden watering schedule.  
Your IP Address will be shown in the form of QR Code in the OLED display, make sure your smartphone/laptop is connected to the same Wi-Fi network as the Orange Pi Zero. 

- Computer: Orange Pi Zero
- OS: Armbian Bionic/Armbian Buster
- Language: Rust
- Database: SQLite

Steps to get this running:
1. Install [Armbian Bionic or Armbian Buster](https://www.armbian.com/orange-pi-zero/) for Orange Pi Zero, burn the image to SD card with [Etcher](https://www.balena.io/etcher/)
2. Log into Orange Pi Zero with USB to TTL converter such as CH340 or CP2102 (Use `screen /dev/ttyUSB0 115200` for Linux or `Putty` for Windows) or through router (use `ssh root@<opi_ip_address>` for Linux or `Putty` for Windows). Password is 1234
3. Configure Wi-Fi with `nmtui`
```sh
nmtui
```
4. Update apt
```sh
apt update
```
5. Install nginx and mosquitto (systemd will be run automatically)
```sh
apt install nginx mosquitto mosquitto-clients libmosquitto-dev
```
6. Configure mosquitto to use websockets
```sh
echo $'port 1883\nlistener 9001\nprotocol websockets' > /etc/mosquitto/conf.d/websockets.conf
```
7. Configure timezone
```sh
timedatectl set-timezone Asia/Jakarta
apt install ntpdate
ntpdate -u 0.id.pool.ntp.org && hwclock -w
```
7. Install Rust via Rustup
```sh
curl https://sh.rustup.rs -sSf | sh
```
8. Clone this repo and compile the code
```sh 
git clone https://github.com/vmasdani/smart-garden.git &&\
cd smart-garden &&\
cargo build --release
```
9. Enable `/dev/i2c-1` through `armbian-config`
```sh
armbian-config
```
10. Copy the contents of the `www` folder to `/var/www/html`
```sh
sudo cp www/* /var/www/html
```
11. Add `target/release/smart_garden` to `/etc/systemd/system/smart-garden.service`  

Use your favorite text editor (nano, vim, etc.)  

`smart-garden.service`  
  
```sh
[Unit]
Description=Smart Garden Service
After=network.target network-online.target mosquitto.service nginx.service 
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=root
ExecStart=/root/smart-garden/target/release/smart_garden

[Install]
WantedBy=multi-user.target
```
12. Run the systemd script
```sh
systemctl start smart-garden
systemctl enable smart-garden
```

Congratulations, the smart garden IoT system is now active!
