package dk.kvalitetsit.hjemmebehandling.migrations;

import ca.uhn.fhir.context.FhirContext;
import ca.uhn.fhir.rest.client.api.IGenericClient;
import org.hl7.fhir.r4.model.Bundle;
import org.hl7.fhir.r4.model.Patient;
import org.hl7.fhir.r4.model.Questionnaire;
import org.hl7.fhir.r4.model.Reference;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.autoconfigure.elasticsearch.ElasticsearchRestClientAutoConfiguration;
import org.springframework.boot.autoconfigure.jdbc.DataSourceAutoConfiguration;
import org.springframework.http.HttpEntity;
import org.springframework.http.HttpHeaders;
import org.springframework.web.client.RestTemplate;

@SpringBootApplication(exclude = {DataSourceAutoConfiguration.class, ElasticsearchRestClientAutoConfiguration.class})
public class Main implements ApplicationRunner {

    private static final Logger LOG = LoggerFactory.getLogger(Main.class);
    @Value("${serverBase:}")
    String serverBase;
    @Value("${patientContactDefaultOrganization:}")
    String patientContactDefaultOrganization;

    @Value("${diasExitUrl:}")
    String diasExitUrl;

    public static void main(String[] args) {
        SpringApplication.run(Main.class, args);
    }

    @Override
    public void run(ApplicationArguments args) throws Exception {
        if (serverBase == null || "".equals(serverBase)) {
            LOG.error("'serverBase' must be specified!");
            exit();
        }

        if (patientContactDefaultOrganization == null || "".equals(patientContactDefaultOrganization)) {
            LOG.error("'patientContactOrganization' must be specified!");
            exit();
        }

        FhirContext ctx = FhirContext.forR4();
        IGenericClient client = ctx.newRestfulGenericClient(serverBase);

        // update patient contact(s)
        setDefaultOrganizationForPatientContacts(client);

        // update questionnaire(s)
        updateCallToActionStructureForQuestionnaires(client);

        LOG.info("All done!");
        exit();
    }

    private void setDefaultOrganizationForPatientContacts(IGenericClient client) {
        // get patients
        Bundle results = client.search()
                .forResource(Patient.class)
                .returnBundle(Bundle.class)
                .execute();

        LOG.info(String.format("Starting migrate for patient contact. Setting default organization to: '%s'", patientContactDefaultOrganization));

        results.getEntry().forEach(bundleEntryComponent -> {
            Patient patient = (Patient) bundleEntryComponent.getResource();

            for (Patient.ContactComponent contactComponent : patient.getContact()) {
                if (!contactComponent.hasOrganization()) {
                    LOG.info(String.format("Updating contact for patient: '%s'", patient.getNameFirstRep().getNameAsSingleString()));
                    contactComponent.setOrganization(new Reference(patientContactDefaultOrganization));
                    client.update().resource(patient).execute();
                }
            }
        });
        LOG.info("Done");
    }

    private void updateCallToActionStructureForQuestionnaires(IGenericClient client) {
        // get questionnaires
        Bundle questionnaireResult = client.search()
                .forResource(Questionnaire.class)
                .returnBundle(Bundle.class)
                .execute();

        LOG.info("Starting migrate for Questionnaire.");

        questionnaireResult.getEntry().forEach(bundleEntryComponent -> {
            Questionnaire questionnaire = (Questionnaire) bundleEntryComponent.getResource();

            for (Questionnaire.QuestionnaireItemComponent item : questionnaire.getItem()) {
                if (item.getType() == Questionnaire.QuestionnaireItemType.GROUP) {
                    if (item.hasItem() && item.getItemFirstRep().getType() == Questionnaire.QuestionnaireItemType.DISPLAY && item.getItemFirstRep().getLinkId().startsWith("call-to-action")) {
                        // this is the 'old-style' call-to-action question-item

                        // change to 'new-style'
                        LOG.info(String.format("Updating call-to-action structure for questionnaire: %s", questionnaire.getId()));
                        item.setType(Questionnaire.QuestionnaireItemType.DISPLAY);
                        item.setLinkId("call-to-action");
                        item.setText(item.getItemFirstRep().getText());
                        item.setEnableWhen(item.getItemFirstRep().getEnableWhen());
                        item.getItem().clear();

                        client.update().resource(questionnaire).execute();
                    }
                }
            }
        });
        LOG.info("Done");
    }

    private void exit() {
        if (diasExitUrl != null && !"".equals(diasExitUrl)) {
            // Calling quit on istio sidecar proxy (DIAS)
            HttpHeaders headers = new HttpHeaders();
            HttpEntity<String> request = new HttpEntity<String>(headers);

            RestTemplate template = new RestTemplate();
            template.postForLocation(diasExitUrl, request);
        }

        System.exit(0);
    }
}