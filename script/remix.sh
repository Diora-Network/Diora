apt-get -y install nodejs
apt-get -y install npm
npm install -g remixd
remixd -s ./contracts/ --remix-ide http://remix.ethereum.org/
