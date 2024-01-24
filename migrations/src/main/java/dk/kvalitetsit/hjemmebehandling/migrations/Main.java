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

@SpringBootApplication(exclude = {DataSourceAutoConfiguration.class, ElasticsearchRestClientAutoConfiguration.class})
public class Main implements ApplicationRunner {

    private static final Logger LOG = LoggerFactory.getLogger(Main.class);
    @Value("${serverBase:}")
    String serverBase;
    @Value("${patientContactOrganization:}")
    String patientContactOrganization;

    public static void main(String[] args) {
        SpringApplication.run(Main.class, args);
    }

    @Override
    public void run(ApplicationArguments args) throws Exception {
        if (serverBase == null || "".equals(serverBase)) {
            LOG.error("'serverBase' must be specified!");
            System.exit(0);
        }

        LOG.info(String.format("Starting migrate for patient contact. Setting organization to: '%s'", patientContactOrganization));
        if (patientContactOrganization == null || "".equals(patientContactOrganization)) {
            LOG.error("'patientContactOrganization' must be specified!");
            System.exit(0);
        }
        FhirContext ctx = FhirContext.forR4();
        IGenericClient client = ctx.newRestfulGenericClient(serverBase);

        // get patients
        Bundle results = client.search()
                .forResource(Patient.class)
                .returnBundle(Bundle.class)
                .execute();

        LOG.info(String.format("Found %s patients", results.getEntry().size()));

        results.getEntry().forEach(bundleEntryComponent -> {
            Patient patient = (Patient) bundleEntryComponent.getResource();

            if (patient.getContact().size() == 1 ) {
                LOG.info(String.format("Updating contact for patient: %s", patient.getNameFirstRep().getNameAsSingleString()));
                patient.getContactFirstRep().setOrganization(new Reference(patientContactOrganization));
                client.update().resource(patient).execute();
            }
            else if (patient.getContact().size() > 1 ) {
                LOG.warn(String.format("Patient har flere kontakter: %s", patient.getNameFirstRep().getNameAsSingleString()));
            }
            // if no contact (size=0) nothing to update
        });

        // get patients
        Bundle questionnaireResult = client.search()
                .forResource(Questionnaire.class)
                .returnBundle(Bundle.class)
                .execute();
        
        LOG.info(String.format("Found %s questionnaires", questionnaireResult.getEntry().size()));

        questionnaireResult.getEntry().forEach(bundleEntryComponent -> {
            Questionnaire questionnaire = (Questionnaire) bundleEntryComponent.getResource();

            for (Questionnaire.QuestionnaireItemComponent item : questionnaire.getItem()) {
                if (item.getType() == Questionnaire.QuestionnaireItemType.GROUP) {
                    if (item.hasItem() && item.getItemFirstRep().getType() == Questionnaire.QuestionnaireItemType.DISPLAY && item.getItemFirstRep().getLinkId().startsWith("call-to-action")) {
                        // this is the 'old-style' call-to-action question-item

                        // change to 'new-style'
                        LOG.info(String.format("Updating call-to-action for questionnaire: %s", questionnaire.getId()));
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

;
        LOG.info("Done!");
        System.exit(0);
    }
}