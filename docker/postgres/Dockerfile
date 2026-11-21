# syntax=docker/dockerfile:1
#
# Image PostgreSQL de PPS : image officielle Alpine (la moins exposée aux
# failles connues parmi les variantes officielles), épinglée par empreinte.
#
# On retire gosu, utilisé seulement pour abandonner les droits root au
# démarrage : c'est le seul composant de l'image signalé avec des failles
# hautes ou critiques (bibliothèque standard Go ancienne). Le conteneur
# tourne directement sous l'utilisateur postgres, sans jamais être root.

FROM postgres:18.6-alpine3.24@sha256:9ba53c7d41ea093baf5c1ba635ba5434c4f979880527d49bb7d9237d3193085b
RUN rm -f /usr/local/bin/gosu
USER postgres
