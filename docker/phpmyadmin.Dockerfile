FROM phpmyadmin/phpmyadmin:5.2.0

ARG UNAGI_PASSWORD
RUN [ "${UNAGI_PASSWORD}" != '' ]
ENV UNAGI_PASSWORD=$UNAGI_PASSWORD

ENV PMA_HOST=/cloudsql/icfpc-primary:asia-northeast1:primary
ENV PMA_SOCKET=/cloudsql/icfpc-primary:asia-northeast1:primary
ENV PMA_USER=root
ENV PMA_PASSWORD=$UNAGI_PASSWORD
