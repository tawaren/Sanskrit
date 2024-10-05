FROM mcr.microsoft.com/devcontainers/rust:latest

# Copy the scripts into the container
COPY ./sp1_prepare.sh /script/
COPY ./sp1_install.sh /script/

RUN chmod +x /script/sp1_prepare.sh
RUN chmod +x /script/sp1_install.sh

# Run the first script and set up PATH
RUN /bin/bash -c '/script/sp1_prepare.sh'
RUN /bin/bash -c '/script/sp1_install.sh'

