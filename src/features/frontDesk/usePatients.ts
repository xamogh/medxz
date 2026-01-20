import { useEffect, useMemo, useState } from "react";
import type { NewPatientInput, Patient } from "./patientStore";
import { createPatient, loadPatients, patientMatchesQuery, savePatients } from "./patientStore";

export function usePatients(organizationId: string) {
  const [patients, setPatients] = useState<Patient[]>(() => loadPatients(organizationId));

  useEffect(() => {
    savePatients(organizationId, patients);
  }, [organizationId, patients]);

  const patientsById = useMemo(() => {
    return new Map(patients.map((patient) => [patient.id, patient]));
  }, [patients]);

  function addPatient(input: NewPatientInput) {
    const patient = createPatient(organizationId, input);
    setPatients((current) => [patient, ...current]);
    return patient;
  }

  function findPatients(query: string) {
    return patients.filter((patient) => patientMatchesQuery(patient, query));
  }

  return {
    patients,
    patientsById,
    addPatient,
    findPatients,
  };
}
