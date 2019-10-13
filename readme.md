# Smart Garden  

A full-stack local web-app in Orange Pi Zero and Rust to manage home garden watering schedule.  
Your IP Address will be shown in the form of QR Code in the OLED display, make sure your smartphone/laptop is connected to the same Wi-Fi network as the Orange Pi Zero. 

- Computer: Orange Pi Zero
- OS: Armbian Bionic
- Language: Rust
- Database: SQLite

Steps to get this running:
- Install [Armbian Bionic](https://www.armbian.com/orange-pi-zero/) for Orange Pi Zero with [Etcher](https://www.balena.io/etcher/)
- Log into Orange Pi Zero with USB to TTL converter such as CH340 or CP2102, or through router
- Configure Wi-Fi with `nmtui`
- Install curl
```sh
sudo apt install curl
```
- Install Rust
```sh
curl https://sh.rustup.rs -sSf | sh
```
- Clone this repo and compile the code
```sh 
git clone https://github.com/vmasdani/smart-garden.git &&\
cd smart-garden &&\
cargo build
```
- Move `db` and `www` directory to `target/debug/`
```sh
cp db www target/debug
```
