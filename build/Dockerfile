FROM alpine:3
RUN apk add curl

ARG hapi_server_base_url
ENV hapi_server_base_url=$hapi_server_base_url
WORKDIR /hapi-server-initializer
COPY ./compose/hapi-server-initializer/init-infektionsmedicinsk.sh ./init-infektionsmedicinsk.sh

COPY ./compose/hapi-server-initializer/careplan-infektionsmedicinsk-1.xml /careplan-infektionsmedicinsk-1.xml
COPY ./compose/hapi-server-initializer/careplan-infektionsmedicinsk-2.xml ./careplan-infektionsmedicinsk-2.xml

COPY ./compose/hapi-server-initializer/organization-infektionsmedicinsk.xml ./organization-infektionsmedicinsk.xml

COPY ./compose/hapi-server-initializer/plandefinition-infektionsmedicinsk-1.xml ./plandefinition-infektionsmedicinsk-1.xml

COPY ./compose/hapi-server-initializer/patient-1.xml ./patient-1.xml
COPY ./compose/hapi-server-initializer/patient-2.xml ./patient-2.xml

COPY ./compose/hapi-server-initializer/questionnaire-infektionsmedicinsk-1.xml ./questionnaire-infektionsmedicinsk-1.xml

COPY ./compose/hapi-server-initializer/questionnaireresponse-infektionsmedicinsk-1.xml ./questionnaireresponse-infektionsmedicinsk-1.xml
COPY ./compose/hapi-server-initializer/questionnaireresponse-infektionsmedicinsk-2.xml ./questionnaireresponse-infektionsmedicinsk-2.xml
COPY ./compose/hapi-server-initializer/questionnaireresponse-infektionsmedicinsk-3.xml ./questionnaireresponse-infektionsmedicinsk-3.xml
COPY ./compose/hapi-server-initializer/questionnaireresponse-infektionsmedicinsk-4.xml ./questionnaireresponse-infektionsmedicinsk-4.xml

COPY ./compose/hapi-server-initializer/searchparameter-careplan-satisfied-until.xml ./searchparameter-careplan-satisfied-until.xml
COPY ./compose/hapi-server-initializer/searchparameter-examination-status.xml ./searchparameter-examination-status.xml
COPY ./compose/hapi-server-initializer/searchparameter-organization-careplan.xml ./searchparameter-organization-careplan.xml
COPY ./compose/hapi-server-initializer/searchparameter-organization-plandefinition.xml ./searchparameter-organization-plandefinition.xml
COPY ./compose/hapi-server-initializer/searchparameter-organization-questionnaireresponse.xml ./searchparameter-organization-questionnaireresponse.xml
COPY ./compose/hapi-server-initializer/searchparameter-organization-questionnaire.xml ./searchparameter-organization-questionnaire.xml

ENTRYPOINT ["/hapi-server-initializer/init-infektionsmedicinsk.sh"]
