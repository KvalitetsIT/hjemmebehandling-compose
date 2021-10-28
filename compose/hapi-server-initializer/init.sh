#!/bin/sh

echo 'Installing curl ...';
apk add curl;

echo 'Waiting for hapi-server to be ready ...';
curl -o /dev/null --retry 5 --retry-max-time 40 --retry-connrefused http://hapi-server:8080

echo 'Initializing hapi-server ...';

# Using PUT allows us to control the resource id's.
curl -i -X PUT -d '@/hapi-server-initializer/patient-1.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/Patient/patient-1?_format=xml

curl -i -X PUT -d '@/hapi-server-initializer/questionnaire-1.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/Questionnaire/questionnaire-1?_format=xml
curl -i -X PUT -d '@/hapi-server-initializer/questionnaire-2.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/Questionnaire/questionnaire-2?_format=xml

curl -i -X PUT -d '@/hapi-server-initializer/questionnaireresponse-1.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/QuestionnaireResponse/questionnaireresponse-1?_format=xml
curl -i -X PUT -d '@/hapi-server-initializer/questionnaireresponse-2.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/QuestionnaireResponse/questionnaireresponse-2?_format=xml
curl -i -X PUT -d '@/hapi-server-initializer/questionnaireresponse-3.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/QuestionnaireResponse/questionnaireresponse-3?_format=xml

curl -i -X PUT -d '@/hapi-server-initializer/careplan-1.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/CarePlan/careplan-1?_format=xml

curl -i -X PUT -d '@/hapi-server-initializer/plandefinition-1.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/PlanDefinition/plandefinition-1?_format=xml

echo 'Done initializing hapi-server!';