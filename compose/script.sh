curl 'http://localhost:4001/api/proxy/v1/questionnaire' -X POST -H 'User-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0' -H 'Accept: */*' -H 'Accept-Language: da-DK,da;q=0.5' -H 'Accept-Encoding: gzip, deflate, br, zstd' -H 'Referer: http://localhost:4001/questionnaires/create' -H 'Content-Type: application/json' -H 'Origin: http://localhost:4001' -H 'Connection: keep-alive' -H 'Cookie: JSESSIONID=F758211F5B2D72BF41C685B9553AC248; Idea-b1bc8483=86231699-4996-47fc-acc6-ce73c5e43745' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'Priority: u=4' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' --data-raw '{
 "questionnaire": {
    "title": "Post-operativ Sundhedsovervågning",
    "status": "ACTIVE",
    "questions": [
      {
        "linkId": "9e976415-db0b-4248-a9cc-fb660f18bbd2",
        "text": "Oplever du feber eller kuldegysninger?",
        "abbreviation": "feber_kuldegysninger",
        "helperText": "Angiv om du har haft temperaturstigninger eller kuldegysninger.",
        "questionType": "BOOLEAN",
        "options": [],
        "enableWhen": [],
        "thresholds": [
          {
            "questionId": "9e976415-db0b-4248-a9cc-fb660f18bbd2",
            "type": "ABNORMAL",
            "valueBoolean": true,
            "valueOption": "true"
          },
          {
            "questionId": "9e976415-db0b-4248-a9cc-fb660f18bbd2",
            "type": "NORMAL",
            "valueBoolean": false,
            "valueOption": "false"
          }
        ],
        "measurementType": {},
        "subQuestions": []
      },
      {
        "linkId": "c5db8034-83b3-4d8c-bad8-35637cdc0761",
        "text": "Hvad er din aktuelle kropstemperatur?",
        "abbreviation": "kropstemperatur",
        "helperText": "Angiv din temperatur i grader Celsius.",
        "questionType": "QUANTITY",
        "options": [],
        "enableWhen": [],
        "thresholds": [],
        "measurementType": {
          "system": "urn:oid:1.2.208.176.2.1",
          "code": "NPU08676",
          "display": "Legeme temp.;Pt"
        },
        "subQuestions": []
      },
      {
        "linkId": "caa7e144-c41e-45db-9c33-bfd16c66294d",
        "text": "Oplever du nogen smerte eller ubehag?",
        "abbreviation": "smerte_ubehag",
        "helperText": "Vælg alle symptomer, du oplever.",
        "questionType": "GROUP",
        "options": [],
        "enableWhen": [],
        "thresholds": [],
        "measurementType": {},
        "subQuestions": [
          {
            "linkId": "5f9beb55-f709-4ffa-b291-466ea7268251",
            "questionType": "QUANTITY",
            "enableWhen": [],
            "thresholds": [],
            "measurementType": {
              "system": "urn:oid:1.2.208.176.2.1",
              "code": "NPU19748",
              "display": "C-reaktivt protein [CRP];P"
            }
          },
          {
            "linkId": "4298d2c3-9559-46f1-8bbb-fe3719ae5598",
            "questionType": "QUANTITY",
            "enableWhen": [],
            "thresholds": [],
            "measurementType": {
              "system": "urn:oid:1.2.208.176.2.1",
              "code": "DNK05472",
              "display": "Blodtryk systolisk;Arm"
            }
          }
        ]
      },
      {
        "linkId": "533fdf9d-136c-4a38-bd06-972d28f1c1d6",
        "text": "Tager du nogen medicin?",
        "abbreviation": "medicin_status",
        "helperText": "Vælg din nuværende medicinstatus.",
        "questionType": "CHOICE",
        "options": [
          {
            "option": "Ja",
            "comment": "Du tager aktuelt foreskrevet medicin."
          },
          {
            "option": "Nej",
            "comment": "Du tager ikke nogen medicin i øjeblikket."
          }
        ],
        "enableWhen": [],
        "thresholds": [],
        "measurementType": {},
        "subQuestions": []
      },
      {
        "linkId": "3a66d0fa-7e89-4a68-89de-1501a3eadbb4",
        "text": "Hvordan vil du vurdere din smerte på en skala fra 1 til 10?",
        "abbreviation": "smerte_skala",
        "helperText": "1 er minimal smerte, 10 er den værst tænkelige smerte.",
        "questionType": "CHOICE",
        "options": [
          {
            "option": "1",
            "comment": "Minimal smerte"
          },
          {
            "option": "5",
            "comment": "Moderat smerte"
          },
          {
            "option": "10",
            "comment": "Svær smerte"
          }
        ],
        "enableWhen": [],
        "thresholds": [
          {
            "questionId": "3a66d0fa-7e89-4a68-89de-1501a3eadbb4",
            "type": "NORMAL",
            "valueOption": "1"
          },
          {
            "questionId": "3a66d0fa-7e89-4a68-89de-1501a3eadbb4",
            "type": "ABNORMAL",
            "valueOption": "5"
          },
          {
            "questionId": "3a66d0fa-7e89-4a68-89de-1501a3eadbb4",
            "type": "CRITICAL",
            "valueOption": "10"
          }
        ],
        "measurementType": {},
        "subQuestions": []
      },
      {
        "linkId": "52f37967-2399-4daa-9d28-d407bfe0fd3c",
        "text": "Har du bemærket nogen hævelse på operationsstedet?",
        "abbreviation": "operationssted_haevning",
        "helperText": "Angiv om der er synlig hævelse.",
        "questionType": "QUANTITY",
        "options": [],
        "enableWhen": [],
        "thresholds": [],
        "measurementType": {
          "system": "urn:oid:1.2.208.176.2.1",
          "code": "NPU19748",
          "display": "C-reaktivt protein [CRP];P"
        },
        "subQuestions": []
      },
      {
        "linkId": "a3dd09af-dde8-42e9-aafe-0fb68b7a9267",
        "text": "Oplever du åndenød?",
        "abbreviation": "aandenod",
        "helperText": "Angiv om du oplever åndenød eller problemer med vejrtrækningen.",
        "questionType": "QUANTITY",
        "options": [],
        "enableWhen": [],
        "thresholds": [],
        "measurementType": {
          "system": "urn:oid:1.2.208.176.2.1",
          "code": "DNK05472",
          "display": "Blodtryk systolisk;Arm"
        },
        "subQuestions": []
      }
    ],
    "callToAction": {
      "linkId": "afb74120-57b4-4aea-806c-fba16de3d271",
      "text": "Venligst udfyld spørgeskemaet for bedre overvågning af din sundhed. Hvis nogen symptomer forværres, søg straks lægehjælp.",
      "questionType": "DISPLAY",
      "enableWhen": [
        {
          "answer": {
            "linkId": "9e976415-db0b-4248-a9cc-fb660f18bbd2",
            "value": "true",
            "answerType": "BOOLEAN"
          },
          "operator": "EQUAL"
        }
      ]
    }
  }
}'





curl 'http://localhost:4001/api/proxy/v1/questionnaire' -X POST -H 'User-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0' -H 'Accept: */*' -H 'Accept-Language: da-DK,da;q=0.5' -H 'Accept-Encoding: gzip, deflate, br, zstd' -H 'Referer: http://localhost:4001/questionnaires/create' -H 'Content-Type: application/json' -H 'Origin: http://localhost:4001' -H 'Connection: keep-alive' -H 'Cookie: JSESSIONID=F758211F5B2D72BF41C685B9553AC248; Idea-b1bc8483=86231699-4996-47fc-acc6-ce73c5e43745' -H 'Sec-Fetch-Dest: empty' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Site: same-origin' -H 'Priority: u=4' -H 'Pragma: no-cache' -H 'Cache-Control: no-cache' --data-raw '{
 "questionnaire": {
    "title": "Daglig Velværesovervågning",
    "status": "ACTIVE",
    "questions": [
      {
        "linkId": "1a2b3c4d-5678-9101-1121-314151617181",
        "text": "Har du følt dig træt eller udmattet i dag?",
        "abbreviation": "traet_eller_udmattet",
        "helperText": "Angiv om du har følt dig unormalt træt.",
        "questionType": "BOOLEAN",
        "options": [],
        "enableWhen": [],
        "thresholds": [
          {
            "questionId": "1a2b3c4d-5678-9101-1121-314151617181",
            "type": "ABNORMAL",
            "valueBoolean": true,
            "valueOption": "true"
          },
          {
            "questionId": "1a2b3c4d-5678-9101-1121-314151617181",
            "type": "NORMAL",
            "valueBoolean": false,
            "valueOption": "false"
          }
        ],
        "measurementType": {},
        "subQuestions": []
      },
      {
        "linkId": "22334455-6677-8899-aabb-ccddeeff0011",
        "text": "Hvad er din vægt i dag?",
        "abbreviation": "daglig_vaegt",
        "helperText": "Angiv din vægt i kilogram.",
        "questionType": "QUANTITY",
        "options": [],
        "enableWhen": [],
        "thresholds": [],
        "measurementType": {
          "system": "urn:oid:1.2.208.176.2.1",
          "code": "NPU00302",
          "display": "Legemsvægt;Pt"
        },
        "subQuestions": []
      },
      {
        "linkId": "3f4g5h6j-7890-1234-5678-901234567890",
        "text": "Har du oplevet nogen humørsvingninger?",
        "abbreviation": "humorsvingninger",
        "helperText": "Angiv om du har oplevet pludselige ændringer i dit humør.",
        "questionType": "CHOICE",
        "options": [
          {
            "option": "Ja",
            "comment": "Du har oplevet humørsvingninger."
          },
          {
            "option": "Nej",
            "comment": "Du har ikke bemærket nogen ændringer."
          }
        ],
        "enableWhen": [],
      	"thresholds": [
      		{
      			"questionId": "3f4g5h6j-7890-1234-5678-901234567890",
      			"type": "NORMAL",
      			"valueOption": "Nej"
      		},
      		{
      			"questionId": "3f4g5h6j-7890-1234-5678-901234567890",
      			"type": "ABNORMAL",
      			"valueOption": "Ja"
      		}
      	]
        "measurementType": {},
        "subQuestions": []
      },
      {
        "linkId": "98765432-1abc-def0-2345-6789abcdef01",
        "text": "Hvordan vil du vurdere din generelle energi i dag?",
        "abbreviation": "energi_niveau",
        "helperText": "1 er meget lav energi, 10 er maksimal energi.",
        "questionType": "CHOICE",
        "options": [
          {
            "option": "1",
            "comment": "Meget lav energi"
          },
          {
            "option": "5",
            "comment": "Moderat energi"
          },
          {
            "option": "10",
            "comment": "Høj energi"
          }
        ],
        "enableWhen": [],
      	"thresholds": [
      		{
      			"questionId": "98765432-1abc-def0-2345-6789abcdef01",
      			"type": "NORMAL",
      			"valueOption": "1"
      		},
      		{
      			"questionId": "98765432-1abc-def0-2345-6789abcdef01",
      			"type": "ABNORMAL",
      			"valueOption": "5"
      		},
      		{
      			"questionId": "98765432-1abc-def0-2345-6789abcdef01",
      			"type": "CRITICAL",
      			"valueOption": "10"
      		}
      	]

        "measurementType": {},
        "subQuestions": []
      }
    ],
    "callToAction": {
      "linkId": "5678abcd-1234-efgh-5678-ijklmnop9012",
      "text": "Udfyld venligst dette spørgeskema dagligt for at overvåge dit generelle velvære.",
      "questionType": "DISPLAY",
      "enableWhen": []
    }
  }
}'
