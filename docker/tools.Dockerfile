FROM ubuntu:22.04
ADD bin/apt-install /usr/local/bin/apt-install
RUN apt-install openssl make
