apt-get -y install nodejs
apt-get -y install npm
npm install -g @remix-project/remixd
remixd -s ./contracts/ --remix-ide http://remix.ethereum.org/
