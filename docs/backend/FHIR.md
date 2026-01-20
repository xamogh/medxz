# FHIR Strategy (Interoperability)

FHIR (HL7 Fast Healthcare Interoperability Resources) is the dominant standard for exchanging clinical data between systems. It defines JSON “Resources” such as `Patient`, `Encounter`, `Observation`, `Condition`, `MedicationRequest`, and document resources like `DocumentReference`.

## Why it matters for this project
- Hospitals and many labs/partners expect FHIR for integration.
- AI features are safer when the underlying clinical facts have standard semantics/codings.
- Reporting/export becomes easier when concepts align to common models.

## The key mismatch to handle
FHIR resources are **state snapshots** (what a `Patient` looks like now).  
This project needs **append-only, audit-first operations** (what changed, by whom, when, and why).

So the usual approach is:
- Keep an **operation log** for safety + sync + audit.
- Maintain **local read models** for UX.
- Generate **FHIR snapshots** (export) from read models (or by replaying ops).

## Options (and recommendation)

### Option A — “FHIR-aware internal model” (recommended for MVP)
Keep internal domain tables and op types, but align core concepts with FHIR:
- Use FHIR-style primitives in our internal schema:
  - `Identifier` (system/value)
  - `CodeableConcept` (coding: system/code/display)
  - `Reference` (type + id)
  - `Quantity` (value/unit/system/code; use UCUM for units)
- Use familiar resource naming for entities:
  - `Patient`, `Encounter`, `Observation`, `Condition`, `Medication*`, `ClinicalNote`, `Attachment`
- Store clinical codings early (even if we don’t validate value sets yet):
  - Vitals/labs: LOINC
  - Diagnoses: SNOMED / ICD-10 (depending on region)
  - Meds: RxNorm (or local formulary mapping)

**Pros:** minimal overhead now, no rewrite later, best fit for offline-first + event-ish ops.  
**Cons:** you still need an export layer later.

### Option B — “FHIR-first storage”
Store canonical records as FHIR JSON blobs (and treat ops as JSON Patch-like edits).

**Pros:** export is “already FHIR”.  
**Cons:** harder offline querying/indexing, harder conflict/merge per domain, and you still need careful audit + revision modeling.

### Option C — “Full FHIR server”
Expose full FHIR REST endpoints and implement profiles/validation/search semantics.

**Pros:** maximum interoperability.  
**Cons:** huge scope; not MVP-compatible.

**Recommendation:** Option A now, then add export/import (Option A → partial Option C) later as needed.

## Concrete plan for Option A
- Keep our sync `Operation` payloads in a stable internal schema.
- Ensure every clinically-coded field uses `{ system, code, display }`.
- Provide an export layer later:
  - `GET /v1/export/fhir` server-side export (Bundle or NDJSON per resource type)
  - Optional: import for patient demographics and meds lists

## Non-goals for MVP
- Full FHIR REST API compliance.
- Terminology server/value set enforcement (we can store codes without validating every constraint).
