#!/bin/sh

echo 'Installing curl ...';
apk add curl;

if [ ! -z $data_dir ]
then
  #echo 'changing dir to: '$data_dir
  cd $data_dir
fi

if [ -z $hapi_server_base_url ]
then
  echo 'hapi-server url not defined. Using default: http://hapi-server:8080'
  hapi_server_base_url=http://hapi-server:8080
fi

echo 'Waiting for hapi-server ('$hapi_server_base_url') to be ready ...';
curl -o /dev/null --retry 5 --retry-max-time 40 --retry-connrefused $hapi_server_base_url

echo 'Initializing hapi-server ...';

function delete {
  echo 'Deleting '$1' ...'

  if [ $(curl -s -o /dev/null -w '%{http_code}' -X DELETE $hapi_server_base_url'/fhir/'$1) -eq '200' ]
  then
    echo 'successfully deleted '$1'!'
  else
    echo 'Nothing to delete ...'
  fi
}

function create {
  echo 'Creating '$2' ...'

  echo $PWD
  # Using PUT allows us to control the resource id's.
  if $(echo $(curl -v -s -o /dev/null -w '%{http_code}' -X PUT -d '@./'$1 -H 'Content-Type: application/fhir+xml' $hapi_server_base_url'/fhir/'$2'?_format=xml') | grep -qE '^20(0|1)$');
  then
    echo 'successfully created '$2'!'
  else
    echo 'error creating object, exiting ...'
    exit 1
  fi
}

delete 'SearchParameter/searchparameter-organization-questionnaireresponse'
delete 'SearchParameter/searchparameter-organization-questionnaire'
delete 'SearchParameter/searchparameter-organization-plandefinition'
delete 'SearchParameter/searchparameter-organization-careplan'
delete 'SearchParameter/searchparameter-examination-status'
delete 'SearchParameter/searchparameter-careplan-satisfied-until'

delete 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-4'
delete 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-3'
delete 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-2'
delete 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-1'

delete 'CarePlan/careplan-infektionsmedicinsk-2'
delete 'CarePlan/careplan-infektionsmedicinsk-1'

delete 'PlanDefinition/plandefinition-infektionsmedicinsk-1'

delete 'Questionnaire/questionnaire-infektionsmedicinsk-1'

delete 'Patient/patient-2'
delete 'Patient/patient-1'

delete 'Organization/organization-infektionsmedicinsk'

create 'organization-infektionsmedicinsk.xml' 'Organization/organization-infektionsmedicinsk'

create 'patient-1.xml' 'Patient/patient-1'
create 'patient-2.xml' 'Patient/patient-2'

create 'questionnaire-infektionsmedicinsk-1.xml' 'Questionnaire/questionnaire-infektionsmedicinsk-1'

create 'plandefinition-infektionsmedicinsk-1.xml' 'PlanDefinition/plandefinition-infektionsmedicinsk-1'

create 'careplan-infektionsmedicinsk-1.xml' 'CarePlan/careplan-infektionsmedicinsk-1'
create 'careplan-infektionsmedicinsk-2.xml' 'CarePlan/careplan-infektionsmedicinsk-2'

create 'questionnaireresponse-infektionsmedicinsk-1.xml' 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-1'
create 'questionnaireresponse-infektionsmedicinsk-2.xml' 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-2'
create 'questionnaireresponse-infektionsmedicinsk-3.xml' 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-3'
create 'questionnaireresponse-infektionsmedicinsk-4.xml' 'QuestionnaireResponse/questionnaireresponse-infektionsmedicinsk-4'

create 'searchparameter-careplan-satisfied-until.xml' 'SearchParameter/searchparameter-careplan-satisfied-until'
create 'searchparameter-examination-status.xml' 'SearchParameter/searchparameter-examination-status'
create 'searchparameter-organization-careplan.xml' 'SearchParameter/searchparameter-organization-careplan'
create 'searchparameter-organization-plandefinition.xml' 'SearchParameter/searchparameter-organization-plandefinition'
create 'searchparameter-organization-questionnaire.xml' 'SearchParameter/searchparameter-organization-questionnaire'
create 'searchparameter-organization-questionnaireresponse.xml' 'SearchParameter/searchparameter-organization-questionnaireresponse'

echo 'Done initializing hapi-server!';
