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
  echo ''
  hapi_server_base_url=http://hapi-server:8080
fi
if [ -z $init_questionnaire_and_plandefinition_infektionsmedicinsk ]
then
  echo 'ENV variable for initializing Questionnaire and PlanDefinition for 'Infektionsmedicinsk Afdeling - Skejby' not defined. Using default: '
  echo '  init_questionnaire_and_plandefinition_infektionsmedicinsk=false'
  echo ''
  init_questionnaire_and_plandefinition_infektionsmedicinsk=false
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
  if $(echo $(curl -s -o /dev/null -w '%{http_code}' -X PUT -d '@./'$1 -H 'Content-Type: application/fhir+xml' $hapi_server_base_url'/fhir/'$2'?_format=xml') | grep -qE '^20(0|1)$');
  then
    echo 'successfully created '$2'!'
  else
    echo 'error creating object, exiting ...'
    curl -X PUT -vvv -d @$1 -H 'Content-Type: application/fhir+xml' "$hapi_server_base_url/fhir/$2?_format=xml"
    exit 1
  fi
}

# Bootstrap all organisations
function bootstrap {
  echo "Bootrapping organisations specified in $1̈́"
  for organisation in `ls  $1`; do
    delete "ValueSet/valueset-$organisation"
    delete "Organization/organization-$organisation"
    create "$1/$organisation/organization.xml" "Organization/organization-$organisation" &&
    create "$1/$organisation/valueset.xml" "ValueSet/valueset-$organisation"
  done
}

function create_searchparameters {

  # Cleanup
  for file in `ls .data/general/searchparamter/ | cut -d "." -f 1`; do
    delete "SearchParameter/searchparameter-$file"
  done


  for file in `ls .data/general/searchparameter/ | cut -d "." -f 1`; do
    create "./data/general/searchparameter/$file.xml" "SearchParameter/searchparameter-$file"
  done
}



if [ $init_test_data = 'true' ]
then
  echo 'Initializing test data too!'

  delete 'Patient/patient-2'
  delete 'Patient/patient-1'

  delete 'QuestionnaireResponse/questionnaireresponse-4'
  delete 'QuestionnaireResponse/questionnaireresponse-3'
  delete 'QuestionnaireResponse/questionnaireresponse-2'
  delete 'QuestionnaireResponse/questionnaireresponse-1'

  delete 'CarePlan/careplan-2'
  delete 'CarePlan/careplan-1'

  delete 'PlanDefinition/plandefinition-2'
  delete 'PlanDefinition/plandefinition-1'

  delete 'Questionnaire/questionnaire-2'
  delete 'Questionnaire/questionnaire-1'

  delete 'Organization/organization-2'
  delete 'Organization/organization-1'
  
  create 'patient-1.xml' 'Patient/patient-1'
  create 'patient-2.xml' 'Patient/patient-2'
  
  create 'organization-1.xml' 'Organization/organization-1'
  create 'organization-2.xml' 'Organization/organization-2'

  create 'questionnaire-1.xml' 'Questionnaire/questionnaire-1'
  create 'questionnaire-2.xml' 'Questionnaire/questionnaire-2'

  create 'plandefinition-1.xml' 'PlanDefinition/plandefinition-1'
  create 'plandefinition-2.xml' 'PlanDefinition/plandefinition-2'

  create 'careplan-1.xml' 'CarePlan/careplan-1'
  create 'careplan-2.xml' 'CarePlan/careplan-2'

  create 'questionnaireresponse-1.xml' 'QuestionnaireResponse/questionnaireresponse-1'
  create 'questionnaireresponse-2.xml' 'QuestionnaireResponse/questionnaireresponse-2'
  create 'questionnaireresponse-3.xml' 'QuestionnaireResponse/questionnaireresponse-3'
  create 'questionnaireresponse-4.xml' 'QuestionnaireResponse/questionnaireresponse-4'
  
fi
  ## Script
  bootstrap ./data/production

create_searchparameters 
delete 'CodeSystem/codesystem-npu-dk'
create './data/general/codesystem-npu-dk.xml' 'CodeSystem/codesystem-npu-dk'

echo 'Done initializing hapi-server!';

echo 'Calling quit on istio sidecar proxy'
curl -X POST http://localhost:15020/quitquitquit
