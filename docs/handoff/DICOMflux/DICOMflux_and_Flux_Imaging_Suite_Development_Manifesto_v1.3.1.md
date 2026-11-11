# DICOMflux and Flux Imaging Suite Development Manifesto

**Version:** 1.3.1  
**Issued:** 9 November 2026, Asia/Kolkata  
**Programme:** DICOM Toolkit & Image Algorithms, Raster Images  
**Audience:** coding agents, developers, maintainers and technical reviewers  
**Purpose:** common engineering rules for the Rust libraries, their desktop and Android bindings and the Java/Swing demonstrator  
**Licensing direction:** MIT for project-owned implementation and documentation, with Raster Images attribution and preservation of applicable third-party notices; external contributions require copyright assignment

## 1. Mission and authority

Build an independently implemented, standards-led Rust DICOM stack and imaging suite that delivers precise results, predictable resource use and familiar APIs across small computers, mobile devices, workstations and servers.

DICOMflux is a fresh, independently implemented Rust DICOM toolkit. The Flux Imaging Suite supplies the Rust counterparts to the compression and resampling capabilities of the Swift Imaging Suite (SIS). Their implementation is the programme's first priority. Windows and Linux on x64 and ARM64, and Android with Kotlin/JNI integration, are primary targets for the libraries. Initial versions of DICOM Capacitor, the DICOM Studio-equivalent technology demonstrator, use Java and Swing over the public Rust interfaces on Windows and Linux. An Android version of DICOM Capacitor is not required. No Android exclusion for DICOM Compressor has been decided. DICOM Camera and Oviyam | Mobile are intended Android consumers of the Rust core. macOS compatibility is secondary. Existing Apple applications retain DICOMkit and SIS as their default, with compile-time Swift bindings for covered Rust comparisons and Voxelia for Apple 3D. DICOMflux-3D remains a discrete Windows/Linux component.

The owner's directions of 8 and 9 November 2026 establish independent MIT implementation; standards currency; DICOMkit/SIS API and behavioural continuity; DICOM-independent imaging libraries; security, performance, bounded memory and concurrency; platform acceleration; a separate 3D component; and a Java/Swing showcase developed alongside useful Rust capabilities. Android is also a primary library and Kotlin application-integration target, using JNI and the Android NDK; an Android DICOM Capacitor application is not required; Android DICOM Compressor remains independently scoped. External contributors must execute the required copyright assignment. These owner directions govern the programme. The engineering recommendations below identify candidate implementation choices without presenting them as separately approved or already qualified.

The words **MUST**, **MUST NOT**, **SHOULD** and **MAY** identify requirements, prohibitions, defaults admitting a documented justification, and optional choices. A module work order identifies the actual capability to build and the applicable clauses. Routine implementation choices within that work order do not need repeated permission. A material departure from the owner's directions or a shared contract requires a recorded decision; continue unaffected work while resolving the affected question.

This is a development rulebook. A module assignment supplies execution scope and repository authority. This document does not itself expand an earlier bounded work order, rename repositories, authorise publication or establish product or clinical qualification.

### Current priorities and implementation recommendations

| Status | Requirement or recommendation |
| --- | --- |
| Owner direction | Build DICOMflux and the Flux Imaging Suite first; keep them useful as independent Rust libraries and command-line tools. |
| Owner direction | Develop the initial DICOM Capacitor in Java and Swing as a real consumer and technology demonstrator of those libraries on Windows and Linux. |
| Owner direction | Windows and Linux on x64 and ARM remain first release requirements. This desktop plan interprets ARM as ARM64/AArch64; 32-bit appliance profiles remain separately scoped. |
| Owner direction | Android is a primary target for DICOMflux, the Flux Imaging Suite and Kotlin application integration through JNI and the Android NDK. No Android version of DICOM Capacitor is required. DICOM Compressor is not excluded from Android by this decision. |
| Owner direction | macOS compatibility is secondary. Preserve the existing DICOMkit/SIS and Voxelia direction and the requested Swift comparison bindings. |
| Owner direction | Release project-owned implementation under MIT, retain the intended commercial product options, and require copyright assignment for accepted external contributions. |
| Engineering recommendation | Start Android device qualification with arm64-v8a; add x86_64 as a distinct emulator/CI profile. Deliver a Kotlin facade, JNI bridge, AAR package and small consumer/instrumentation harness. |
| Engineering recommendation | For desktop, qualify Java 25 LTS, initially using Microsoft Build of OpenJDK, and the standard Foreign Function and Memory API for the Rust C ABI. Pin the actual distribution, update and native target in the implementation work order. |
| Engineering recommendation | Deliver a bounded CPU-based 2D Swing workflow before making an embedded wgpu viewport a release requirement. Start with synchronous native calls on bounded background workers. |

Java/Swing is the selected initial desktop application direction. Android Kotlin/JNI bindings are part of the primary library delivery. Their initial validation consumer is a small harness; this does not add an Android DICOM Capacitor application requirement. The absence of an Android DICOM Capacitor requirement does not exclude compression, decompression, transcoding or resampling library capabilities supported by the assigned Android profile. Android DICOM Compressor is a separate application decision. C# and Swift integrations retain their separately scheduled scope. Java server integration has its own JDK and deployment profile and does not inherit the desktop baseline automatically.

### The commitments every agent inherits

1. Implement the advertised DICOM and image-processing behaviour in our Rust code.

2. Derive interoperable behaviour from verified standards and explicit capability profiles.

3. Use DICOMkit and SIS experience without reproducing their defects or forcing Swift mechanics into Rust.

4. Keep generic imaging independent of DICOM datasets and application workflows.

5. Preserve sample meaning and source provenance through every operation.

6. Make ownership, concurrency, failure and resource consumption explicit.

7. Provide useful portable CPU behaviour before depending on acceleration.

8. Prove correctness with independently justified expectations and honest execution evidence.

9. Keep dependencies, interfaces and documentation proportionate to the delivered capability.

10. Finish small, executable slices before expanding the programme.

## 2. Independent implementation and MIT provenance

### 2.1 What independence means

The shipping DICOM parser, dataset model, writers and protocol implementations, and the portable Flux codec and resampling algorithms, MUST be implemented within this programme. Wrapping, vendoring or silently invoking dicom-rs, DCMTK, GDCM, dcm4che, OpenJPEG, CharLS, libjxl or another equivalent DICOM or compression engine does not fulfil an independent implementation assignment. The same rule applies to fallback executables and remote processing services.

The portable imaging algorithms must be independently implemented. Qualified accelerated paths MAY delegate numerical or image-processing operations to the platform primitives expressly permitted by section 2.3, including vImage resampling, while preserving the specified semantics and the independently implemented portable counterpart. This permission does not authorise an external DICOM engine or complete compression codec.

DICOMkit and SIS are the primary internal design, behavioural and API references. Agents MAY study their architecture, algorithms, tests and performance investigations, after checking the actual source identity and any third-party material within them. The Rust implementation MUST be designed and qualified as Rust. Historical Swift test results do not become Rust execution evidence.

Other implementations MAY be used to understand a problem, compare behaviours, discover edge cases or act as external development-time test oracles. They do not define the standard. Where an implementation disagrees with a normative requirement, investigate the requirement and preserve an explanatory regression case.

### 2.2 Reference and source-use policy

| Reference category | Required treatment |
| --- | --- |
| DICOMkit and SIS | Pin approved specifications and actual source revisions; distinguish owned material from third-party inclusions; record relevant design and test reuse |
| MIT or other compatible permissive source | Reading and comparison are permitted subject to the actual licence; any copied or adapted material requires a specific provenance entry and preserved notices; wholesale engine or codec adoption remains outside this independent-implementation direction |
| Copyleft, proprietary or otherwise restricted source | Use independently written descriptions of concepts and methodology, standards and permitted observable behaviour; do not copy, mechanically translate, transpile or closely re-express protected implementation code into this MIT implementation |
| Standards, tables, dictionaries and test vectors | Record the origin, edition and applicable use or redistribution terms separately; free access does not make the material MIT licensed |
| Unlicensed or ambiguous material | Do not incorporate it until the intended use and rights are resolved; proceed from verified standards and other admitted sources where possible |

MIT permits broad reuse subject to retaining the required copyright and permission notices. A language change does not itself reset the obligations associated with adapted code. These authoring rules are deliberately conservative; they do not assert that reading a copyleft project automatically makes an independently written implementation derivative. [S04, S05]

An agent MUST NOT paste restricted implementation source into a code-generation prompt and ask for an MIT rewrite. Reference notes must describe behaviour, mathematics and trade-offs in original words and trace them to admissible sources. If restricted source has already influenced a proposed contribution, disclose the affected material for review rather than hiding its provenance. Use the description **independent implementation with documented provenance**; do not claim a formal clean-room process unless that process has actually been established and followed.

External implementation-source reuse must be narrowly scoped, specifically identified and consistent with the independent-implementation direction. It must not substitute another engine or codec for this programme's implementation. Record its source, exact revision, licence, affected files, purpose and required notices. Generated code, copied constants and test fixtures receive the same scrutiny as handwritten source.

Project-owned DICOMkit/SIS architecture, algorithmic knowledge, tests and implementation techniques may be carried into Rust. Translation or adaptation of owned source is permitted after checking ownership and any upstream inclusions; document that lineage and design and qualify the result as Rust. The constraints on external implementation reuse do not require us to rediscover wholly project-owned work. A Rust wrapper around an existing Swift engine still does not satisfy the native Rust implementation requirement.

Generic infrastructure dependencies MAY be admitted where they support the implementation rather than substitute for it. Examples include platform bindings, logging interfaces, executor adapters, cryptographic libraries and test tooling. Keep them small, inspect their transitive dependencies and document why they are required. Use maintained cryptographic implementations; this independence objective does not commission bespoke cryptography or TLS primitives.

Maintain source and dependency provenance, required notices and a software bill of materials for releases. The MIT licence on project-owned code does not relicense upstream material or grant rights to unrelated patents.

### 2.3 Permitted platform and rendering components

The programme explicitly permits platform numerical primitives and suitable permissively licensed graphics, UI and rendering projects. DICOMflux-3D may integrate wgpu, egui and other admitted rendering functionality. This permission does not turn an external DICOM engine or complete compression codec into an authorised implementation backend.

Platform primitives, including Accelerate/vDSP/vImage operations, may implement a qualified accelerated path for our image-processing contract alongside the independently implemented portable path. Document the delegated operation, precision, boundary rules, ownership and numerical equivalence. Device kernels and minimal platform glue may use the appropriate supported toolchain; their source and dependencies follow the same provenance rules.

At issue, both wgpu and egui declare MIT OR Apache-2.0. Select and record the applicable licence for pinned versions, retain notices, and inspect their transitive dependencies and bundled fonts, shaders and assets. Other freely available projects are admitted according to their actual terms and suitability; availability alone is not a licence grant. Do not describe incorporated upstream rendering code as independently authored project code. A rendering dependency must not silently supply a DICOM or medical-image codec backend. [S21, S22]

### 2.4 Copyright ownership and commercial distribution

Project-owned Rust implementation, Java bindings, the initial Swing application and documentation follow the MIT direction. Copyright ownership and the licence granted to recipients are separate. MIT permits commercial use and sale subject to its notice condition; it does not reserve commercial exploitation exclusively to Raster Images. Recipients may also redistribute or sell the MIT-covered code. A future commercial edition or alternative licence does not withdraw the permissions already granted for earlier MIT releases. [S04, S27]

External copyrightable contributions MUST have an executed copyright assignment covering the contribution before incorporation. The agreement identifies Raster Images' designated legal rights-holder precisely, the covered work and revisions, the contributor's authority and any necessary employer or other rights-holder authorisation. Retain a durable acceptance record; keep private signing and identity records out of public repositories.

A contributor licence agreement, Developer Certificate of Origin or Signed-off-by line is not automatically a copyright assignment. The required assignment instrument and signing procedure must receive legal review for the relevant jurisdictions, including patent permissions and applicable moral-rights treatment. This manifesto establishes the contribution policy; it is not the legal instrument. [S27, S28]

Assignments cover rights the contributor actually holds. Record original work, permitted adaptations, generated material, fixtures and other assets separately. Third-party code retains its own ownership, notices and licence obligations. Apply the same provenance checks to any Mayam source, tests, artwork, fonts or dependencies proposed for reuse. Mayam experience can inform workflows immediately; importing its code requires checking the actual rights and content.

Use an unmodified MIT outbound licence for project-owned material. Do not add a commercial-use prohibition, subscription condition or assignment demand to downstream MIT use; the assignment policy governs contributions accepted into the maintained project. Publish it before inviting external contributions.

The selected Java runtime is a separately licensed dependency. Microsoft Build of OpenJDK is distributed under GPLv2 with the Classpath Exception and provides freely available updates under its stated policy. Preserve the runtime's applicable licence, notices and source-availability obligations; bundling it does not relicense the runtime as MIT. Verify the exact distribution's terms for the shipped package. The Classpath Exception permits independent application modules under other licences, so the independently authored Java application can retain its MIT licence. This does not authorise copying JDK implementation source into the MIT project. [S29, S40]

For the architectural record, Avalonia's MIT core does not require a paid commercial-use subscription. Its professional tools, premium components and support have separate terms. The initial Java/Swing decision stands on the owner's selected direction and existing team experience. [S26]

## 3. Standards currency with reproducible baselines

### 3.1 Interpret latest correctly

At this document's issue date, the official consolidated DICOM publication inspected is **2026d**. This is an engineering source snapshot, not a permanent target or a blanket conformance claim. DICOM Final Text supplements and corrections take effect when approved; consolidation into a later edition is a subsequent publication step. [S01, S02]

At each module kickoff and release, verify the latest consolidated standard and applicable approved Final Text changes. Record what applies to the module and reconcile changes with its implementation and tests. Working drafts, public-comment documents and ballots are research inputs. Experimental support for them must be explicitly commissioned, distinguishable and excluded from ordinary conformance claims. Never invent an official UID for an experimental feature.

Pin the actual edition, documents, sections and source digests used for engineering. A build MUST NOT silently download changing normative definitions from a moving current URL. Generate dictionaries and other normative data through a repeatable, rights-checked process, and review semantic changes before replacing a baseline.

Once a change is incorporated into a consolidated edition, use the consolidated wording. Keep retired-feature compatibility explicit and separate from preferred new output. Source pinning supports reproducibility; standards-impact review prevents the implementation from freezing indefinitely on obsolete behaviour.

### 3.2 State the supported capability precisely

Relevant DICOM material includes PS3.2 through PS3.8 for conformance, objects, services, encoding, dictionary and networking; PS3.10 through PS3.12 for media; PS3.14 for applicable display functions; PS3.15 for relevant security profiles; PS3.16 for terminology and templates; and PS3.18 for web services. Each work order selects the applicable parts and clauses; this list does not commission all of them at once.

Codec work MUST also identify the applicable JPEG, ISO/IEC or ITU-T specifications, parts, amendments and supported profiles. DICOM encapsulation requirements and the underlying image-coding standard are separate obligations.

Publish capabilities by operation and profile: parsing, preservation, inspection, decoding, encoding, validation and communication. Include the relevant SOP Classes and roles, Transfer Syntaxes, sample representations, media profiles or web transactions. A recognised UID is not a decoded image; parsed syntax is not IOD validation; a working library primitive is not a qualified product service.

DICOM conformance claims refer to supported features and roles, with the required product conformance statement, rather than to an edition label such as "fully DICOM 2026d compliant". [S03]

## 4. Module boundaries and portfolio

The dependency direction MUST remain from DICOM integration towards generic image algorithms. A Flux codec MUST build and operate without DICOMflux, sibling codecs, a GUI, a database or an umbrella runtime. A small neutral image-contract crate is permitted; it must not acquire DICOM semantics or unrelated services. DICOMflux-3D, language bindings and DICOM Capacitor are optional consumers; their dependencies must not flow back into the headless core.

| Module or layer | Responsibility |
| --- | --- |
| DICOMflux foundation | Tags, VRs, values, datasets, dictionaries, bounded parsing/writing, errors and bulk-data access |
| DICOMflux image adapters | Interpret DICOM image metadata, select declared codecs, locate frames, handle encapsulation and construct updated datasets |
| Optional DICOMflux capabilities | Individually scoped networking, web services, media/DICOMDIR, validation, de-identification or other domain functions |
| FluxJ2K | Independently implemented JPEG 2000 and HTJ2K capabilities, with their profiles and directions separately declared |
| FluxJLS | Independently implemented JPEG-LS, with lossless and any bounded-error modes separately declared |
| FluxJXL | Independently implemented JPEG XL; decode, encode and reversible JPEG reconstruction are separate capabilities |
| FluxJLI | The verified SwiftJLI JPEG scope expressed in Rust; the module name must not be treated as evidence of legacy lossless-JPEG support |
| FluxRLE | General PackBits operations and an explicit DICOM RLE frame profile, without a DICOM dataset dependency |
| FluxResample | The agreed fixed-kernel resampling portfolio with precise numerical and boundary semantics |
| Optional suite orchestration | Cross-codec composition and common tools; no mandatory dependency from individual codecs back to this layer |
| Host bindings | Java for desktop DICOM Capacitor and Kotlin/JNI for primary Android integration; later C# and the verified DICOMkit/SIS-compatible Swift facade with build-time core selection |
| Android integration package | Thin Kotlin facade, JNI bridge and ABI-specific Rust libraries in an AAR, with a minimal consumer/instrumentation harness; no Android DICOM Capacitor application requirement; Android DICOM Compressor scope is separate |
| DICOMflux-3D | Discrete Windows/Linux volume and 3D visualisation component using wgpu, with an optional standalone egui harness and admitted rendering dependencies |
| Apple 3D adapter | Connect checked image/volume outputs to the existing Voxelia interfaces through the assigned interoperability contract |
| DICOM Capacitor | Initial Java/Swing DICOM Studio-equivalent technology demonstrator for Windows/Linux x64 and ARM64, consuming public toolkit, codec and presentation interfaces |
| Calling applications | User interaction, application scheduling, storage policy, deployment and product-specific workflows |

A codec may accept external geometry, precision, signedness and component descriptors needed to interpret a raw codestream. That is compatible with DICOM awareness. Patient identity, tags, Transfer Syntax negotiation, association state and DICOM file writing belong outside the generic codec.

The DICOM RLE frame profile is a defined format adapter above generic PackBits. It may encode the standard RLE frame header and byte segments, but does not parse datasets or make patient-level decisions. Existing SIS terminology and profile boundaries must be verified against the assigned reference revisions. [I01]

Keep video, waveform, PDF and other payload support explicit. Metadata inspection and opaque preservation must not be advertised as media decoding or rendering. Additional codec families and advanced volumetric functions enter through their own assignments; the bounded 3D component below does not imply every volumetric capability.

Modules can be independently published from shared workspaces. Repository layout is an execution decision, not a requirement to create a separate repository for every crate. Keep common automation and type contracts consistent without coupling every module's release schedule.

### 4.1 DICOMflux-3D and the replaceable 3D boundary

DICOMflux-3D is the discrete 3D visualisation component for Windows and Linux. Use wgpu for GPU rendering. egui may supply a standalone Rust demonstration or development harness; the renderer MUST remain usable without egui, and the Swing application owns its own interface and event loop. Windows profiles may use qualified Direct3D 12 or Vulkan backends; Linux profiles use qualified Vulkan backends. Other backends require their own profile. [S21, S22]

The initial remit is a bounded viewing capability over admitted image volumes. Select and document the first supported volume profile and viewing operations. MPR, MIP, direct volume rendering, surface extraction, segmentation, registration, fusion and advanced analysis are separate capabilities; the component name does not commission the full Voxelia feature set.

Keep DICOM import, frame selection, geometry validation and construction of a checked volume description usable without a window, egui or GPU initialisation. A volume description identifies sample type and precision, dimensions, strides, physical units, voxel-to-patient mapping, intensity transforms, ownership and provenance. Validate acquisition regularity before treating frames as a regular volume. Derive spatial relationships from the applicable DICOM attributes; do not infer geometry from filenames or use nominal Slice Thickness as a substitute for slice position/spacing. Unsupported mixed, irregular or multidimensional arrangements require an explicit disposition. [S23]

Keep rendering/session state separate from egui widgets. Expose a small versioned application-facing contract for admitted volumes, view/camera requests, transfer functions, capabilities, resource limits, cancellation, presentation/output and errors. Keep wgpu and egui implementation types behind integration boundaries. Render-to-texture or supported callbacks may avoid unnecessary CPU round trips, but sharing and synchronisation must be qualified. Display surfaces must not replace the original quantitative voxel storage.

Query adapter features, texture-format capabilities and resource limits before selecting a path. Bound volume uploads, GPU residency, texture sizes and in-flight work; test unsupported adapters, device loss, allocation failure and cancellation. A GPU-dependent 3D feature may report unavailable while the headless toolkit remains useful. This does not commission a complete software volume renderer. Pin a compatible wgpu/egui integration set and qualify the actual GPU, driver and OS. [S21]

A separately commissioned future project, provisionally called voxelia-rs, may replace this implementation through the same application-facing boundary or an explicit versioned adapter. Preserve small, useful contracts now; do not build a speculative universal rendering framework or a placeholder replacement project.

#### 4.1.1 Swing viewport integration

A useful 2D DICOM Capacitor release MUST be possible without embedded native 3D. First qualify the bounded renderer over checked volume data independently. An optional offscreen render/readback can exercise the Swing presentation path, with its copies and latency reported.

A later dedicated native viewport may use an AWT Canvas and a small JAWT/JNI adapter to obtain supported platform surface information for wgpu. JAWT is a distinct graphics integration concern even when ordinary DICOM/codec calls use FFM. Do not access undocumented AWT peers or assume a Java object reference guarantees that its native window remains alive. JAWT has locking and surface-change rules; wgpu native handles must remain valid for the surface lifetime. [S34, S35]

Qualify native viewport creation, bounds and clipping changes, resize, DPI, focus, popups, annotations, hiding/reparenting, disposal and device loss. Define which layer draws overlays. The Java 25 JAWT specification describes Windows and Linux/X11 structures; qualify X11 and XWayland configurations explicitly. Native Wayland integration requires its own demonstrated host-and-renderer profile. wgpu's backend support alone is insufficient evidence of that interoperability. [S34]

The viewport is an optional adapter with its own acceptance evidence. Do not duplicate the DICOM/codec binding in JNI merely because the graphics adapter needs JNI, and do not make GPU hardware a dependency of headless inspection, codecs or the initial CPU 2D workflow.

### 4.2 Apple 3D integration

Apple applications use Voxelia for 3D, including when DICOM and image-codec plumbing is selected from Rust through Swift bindings. The selected core supplies checked buffers and geometry through a verified Voxelia adapter; changing the core does not imply changing the Apple renderer.

Inspect the assigned Voxelia API and ownership contracts before promising compatibility or shared GPU resources. Reuse buffers where allocation, layout and synchronisation permit it, and report required conversions or copies. Shared fixtures and geometry conventions support comparison, but pixel-identical presentation across renderers is claimed only for a specifically tested operation and tolerance. No port of the full Voxelia stack is commissioned by this manifesto.

## 5. API continuity expressed idiomatically in Rust

### 5.1 Build an API equivalence table first

Before adding a public interface, inspect the assigned DICOMkit/SIS API declarations, contracts, CLI rules and independent-consumer examples at exact revisions. Distinguish an accepted contract from a development-head implementation or an old handoff. Resolve conflicts explicitly; do not guess a public API from a summary.

For each operation record the Swift symbol, intended meaning, Rust symbol, ownership, errors, cancellation, concurrency, result/report fields and any justified divergence. Follow the same conceptual vocabulary and operation families. A spelling conversion alone does not establish equivalence.

The historical SwiftRLE reference names concepts including Image, ImageDescriptor, PlaneDescriptor, ImageDestination, Encoder, Decoder, EncodeOptions, DecodeOptions, ResourceLimits, OperationReport and CodecError. These are useful vocabulary references, not a claim that their current source declarations have been audited by this manifesto. Each agent must confirm the actual assigned baseline. [I01]

### 5.2 Naming and type rules

Preserve recognisable concept names while using Rust conventions: UpperCamelCase types, traits and enum variants; snake_case functions, methods, fields and modules; and explicit ownership through borrowing, owned values and consuming conversions. Fallible construction and operations return typed Result values. Rust's API guidelines supply the naming conventions. [S06]

For example, a verified Swift property meaningfulBits would normally map to meaningful_bits. A verified decode-into operation needs an explicit Rust operation such as decode_into because Rust does not mirror Swift's argument-label overloading. These examples explain the mapping method; they do not freeze uninspected signatures.

Provide a synchronous, runtime-independent execution layer where practical. An async or parallel facade may use the same implementation. Record the deliberate mapping from Swift async behaviour; never block an async method through a hidden semaphore bridge, and never duplicate algorithms merely to offer two calling styles.

Use Rust lifetimes, slices, enums, newtypes and ownership to make invalid states hard to express. Avoid pointer-shaped public APIs where a safe abstraction can express the requirement. Keep unsafe adoption and FFI in clearly identified advanced layers. Do not mechanically reproduce Swift actors, reference counting or class hierarchies when Rust has a better expression of the same guarantee.

Public capabilities, options, error meanings and reports MUST stay consistent across modules. Unsupported options must be rejected, not silently ignored. New shared concepts require one coordinated contract change rather than divergent per-module inventions. Preserve semantics even where an idiomatic Rust spelling differs.

## 6. Image fidelity and DICOM semantics

Correctness and preservation have priority over speed or compression ratio. Stored sample values, modality-transformed values and display pixels are distinct representations. Codecs MUST NOT silently apply windowing, rescale, MONOCHROME1 inversion, palette expansion, colour conversion or an eight-bit display reduction.

A signed 12-bit value held in a 16-bit cell must keep its intended sign and precision. Distinguish preservation of all allocated-cell bits from preservation of meaningful samples. If the DICOM adapter normalises unused bits into a canonical image representation, make the operation explicit and preserve any separately significant information through the applicable compatibility policy.

Require checked descriptions of precision, storage width, byte order, planes, strides, component interpretation and subsampling. Do not infer meaningful precision from observed extrema. Unsupported representations must fail explicitly; the implementation must not silently choose a smaller sample type or a different colour model.

For applicable DICOM functions, explicitly handle sequence/item boundaries, defined and undefined lengths, VR and multiplicity rules, character sets, private creators, unknown elements, encapsulated fragments and frame offsets. A fragment is not generally a frame. Preserve private/unknown content where the declared operation promises preservation, without claiming semantic interpretation. Private-creator reservations have item-local scope. [S09]

Strict validation and any compatibility-reading mode must have distinct policies and diagnostics. Compatibility handling may recognise a documented legacy form; it must not invent missing samples, clip oversized runs or turn a resource-limit failure into successful processing. New output must meet its declared standard profile.

### 6.1 Preservation guarantees

| Guarantee | Required evidence |
| --- | --- |
| Meaningful-sample preservation | Exact logical sample comparison, with representation and interpretation recorded |
| Allocated-cell preservation | Equality of all specified stored bits, including bits outside meaningful sample precision where promised |
| Compressed-bitstream preservation | Recovery of the original compressed bytes for the declared transformation and profile |
| Whole-object byte preservation | Equality of the complete retained or reconstructed DICOM byte sequence |
| Semantic dataset preservation | Defined equivalence of values and structure, acknowledging permitted encoding differences |

These guarantees MUST NOT be conflated. In-memory transcoding describes the processing path and storage use, not a fidelity guarantee. JPEG XL's DICOM JPEG-recompression capability and mathematically lossless J2K/HTJ2K recoding require their own precise profiles and tests. [S10, S11]

Lossy processing must be explicitly selected, bounded by its declared quality/error controls and reflected in DICOM metadata where applicable. Lossless re-encoding of already lossy samples does not restore the original pre-loss information or erase the lossy history. Raw codestream precision and colour characteristics must be consistent with the applicable DICOM interpretation rules. [S10]

Every DICOM image transformation must declare its effects on the surrounding dataset. The DICOMflux adapter must validate and update affected image descriptors, photometric interpretation, Transfer Syntax, geometry and applicable per-frame attributes, together with derivation information, lossy history and instance identifiers as required by the relevant IOD and operation. Do not blindly copy metadata that no longer describes the output or regenerate identifiers without applying the relevant rules. If the effects cannot be determined reliably, reject the transformation or return an explicitly incomplete result for the caller to resolve; never publish it as a complete derived instance.

Acceptance of a DICOM transformation requires evidence for both the pixel operation and dataset consistency. A standalone Flux codec or resampler reports its output representation and transformation parameters; the DICOM adapter owns the corresponding dataset changes.

### 6.2 Resampling

Implement the accepted seven-method portfolio: nearest, bilinear, bicubic, Mitchell-Netravali, Catmull-Rom, Lanczos and area/box. Additional and adaptive kernels are deferred from the initial portfolio.

Each kernel requires explicit parameters, support radius, coordinate and pixel-centre conventions, boundary extension, downsampling/antialiasing policy, intermediate precision, rounding and clipping. Bicubic variants, Lanczos radii and area-versus-box behaviour must be resolved from the authoritative SwiftResample specification. A matching name is insufficient evidence of a matching result.

Separate continuous-image interpolation from discrete-label resampling. Label data must not acquire fractional or invented classes through an inappropriate default. Segmentation and other callers must select a qualified label-preserving operation. Do not silently assume display-oriented gamma or colour processing for quantitative medical samples.

## 7. Memory and buffer ownership

Provide an ergonomic owned layer and an advanced borrowed-buffer layer over the same checked implementation. Every operation MUST state input lifetimes, output capacity, scratch requirements, alignment, aliasing, allocation and completion behaviour.

Borrowed input is immutable unless mutation is explicitly part of the operation. A caller-provided destination requires exclusive write access. Deferred or asynchronous work MUST keep its inputs and outputs valid for the complete execution lifetime, through compiler-enforced scoped borrows or documented owned/retained leases. Detached work requires owned or retained storage. Dropping or cancelling a future, aborting a task or disposing a host wrapper MUST NOT release or reuse storage still reachable by workers or hardware. A borrowed pointer must never outlive its valid scope.

Retained buffers MUST release exactly once through the documented owner after the last consumer and in-flight operation finish. Memory allocated by one library must not be released through another allocator. Pool reuse needs generation/lifetime checks and completion of any hardware work. A pointer address alone is not an ownership proof.

Where a destination follows reserve/write/seal semantics, preserve that lifecycle across adapters. Preflight failures leave an unused destination in its documented reusable state. After writes begin, failure or cancellation must invalidate the begun result or return an explicitly supported partial-result type; it must never publish it as a complete image. Join workers before sealing, invalidation or reuse. The historical SIS lifecycle is a behavioural reference, while Rust must express its guarantees through valid ownership and initialisation rules. [I01]

Every byte covered by a safe Rust byte slice or reference MUST be initialised, including padding inside that view. Uninitialised spare capacity may exist only in a suitable representation such as MaybeUninit or internal raw storage; it MUST NOT be exposed as an ordinary byte slice before initialisation. The reported valid extent defines the completed payload and does not relax Rust's initialisation requirements. [S14]

All arithmetic on dimensions, strides, offsets, capacities, frame counts and decoded sizes MUST be checked before allocation, indexing or pointer construction. Validate the final addressed byte and permitted plane overlap. A 64-bit host is not an excuse to ignore narrower targets or conversion overflow.

Memory accounting includes input buffers, metadata/indexes, decoded samples, output growth, scratch space, caches, queues and all concurrent workers. Shared owners must not be double-counted, and hidden allocations must not be excluded from a footprint claim. Use fallible allocation for input-controlled growth where available; do not promise universal recovery from every allocator or operating-system failure.

Every memory limit MUST name its scope and charging rule: operation, library instance or host-wide admission policy. Distinguish library-controlled allocations, borrowed or retained external backing storage, and measured process resident memory. Charge retained backing allocations according to their actual known extent rather than only the visible slice; report unknown external extents explicitly. A library allocation cap must not be described as a process-wide memory guarantee. Concurrent instances require host-level aggregate admission control where a shared ceiling is promised.

Support memory limits and streaming/back-pressure where the assigned algorithm and input permit them. Document formats or operations requiring random access or global state. A streaming API must not secretly materialise the whole input. File-backed or mapped storage, temporary files and spill are explicit host capabilities; they cannot silently satisfy a memory-only operation.

For cross-codec pipelines, decode directly into a compatible final buffer when the admitted path supports it, retain that owner and let the next stage read it. A require-sharing policy must reject a route needing an undeclared full-image handoff copy. An allow-copy route must report conversions and copies. Report precisely which handoff is copy-free; compressed outputs and genuine bounded algorithm workspace have separate accounting.

## 8. Security and predictable failure

Treat files, streams, network messages, metadata and compressed data as untrusted. Memory safety is one defence; it does not prevent logical corruption, excessive work or denial of service.

Each module MUST enforce finite, configurable limits applicable to its workload: compressed and decoded bytes, dimensions, frames, components, element/item counts, nesting, fragments, output growth, live allocations, queues and cumulative work. Check limits before expensive work and throughout incremental processing. Timeouts and cancellation supplement structural bounds; they do not replace them.

Expected failures use typed errors, with stable categories for invalid arguments, malformed or incomplete input, unsupported capability, incompatible layout, insufficient output capacity, resource limits, cancellation, I/O and internal failure. Incremental need-more-input is distinct from a truncated final input. Error precedence and partial-output rules must be deterministic enough for callers to handle safely.

Untrusted input must not reach unwrap, expect, unchecked conversion, unchecked indexing or an intentional panic through an unvalidated assumption. Safe Rust is the default. Necessary unchecked operations belong inside isolated, reviewed code with a local proof that prior validation establishes every required invariant. Verify pointer, lifetime, alignment, aliasing and initialisation assumptions. Manual Send/Sync implementations require specific justification and concurrency evidence.

Diagnostics must avoid disclosing patient values, image payloads, credentials, raw addresses or identifying paths by default. Supply structured codes, safe field names and offsets where useful. Verbose diagnostics remain bounded and explicitly selected. Test data must be synthetic or rights-cleared; no patient data in public repositories, model prompts or ordinary reports.

No implicit network access, telemetry, shell execution or remote bulk-data retrieval is permitted in a parsing or codec operation. Hosts must explicitly supply and authorise external retrieval, with bounds and location/redirect policies. File and media operations must protect against path traversal and unsafe link resolution. Network modules must bound message sizes, queues and timeouts and use admitted secure transport implementations.

Published modules require a security contact, a vulnerability-handling process and a declared supported-version policy. Assess dependencies and generated/build tooling as well as runtime code. A known issue may be documented with a bounded disposition; it must not be hidden by relabelling a failed test or disabling the relevant validation.

## 9. Concurrency and scaling

### 9.1 The 1.21 gigawatt principle

**DICOMflux does not require 1.21 gigawatts to run.**

A useful headless-core baseline must run CPU-only with modest memory and one worker. Additional cores, vector units and accelerators improve capacity or speed without silently changing required results. Low-resource support is demonstrated with a named target, workload and measured budget, not a marketing adjective. The optional 3D component declares its own graphics requirements and does not impose them on parsing, codecs or other CPU-capable operations.

| Execution profile | Design intent |
| --- | --- |
| Constrained Linux or appliance | Few workers, bounded buffers, selected features, minimal background activity and explicit limits |
| Desktop or mobile | Responsive cancellation, bounded queues, sensible parallelism and explicit host integration |
| Server or big iron | Configurable high throughput, aggregate admission control, fairness and avoidance of oversubscription |
| Optional accelerator | Measured benefit including transfer/synchronisation costs, with device-memory and in-flight-work limits |

Every handle and operation must declare its sharing, mutation and reentrancy rules. Immutable datasets and sealed buffers should support concurrent readers. Mutable decoder/encoder state should be isolated per operation or explicitly synchronised. A blanket "thread-safe" label is insufficient.

Let hosts schedule synchronous work, and offer optional async/executor adapters without requiring a particular runtime in the core. Do not create background workers merely by constructing a parser or inspecting capabilities. Parallel work must respect explicit worker, memory and queue budgets.

Bound nested parallelism. A server processing many frames must not multiply its outer worker count by an unrestricted inner codec pool. Cancellation and timeout handling must join or safely retain outstanding work before returning or releasing buffers. Callback lifetime, thread and reentrancy contracts must be explicit; avoid invoking host callbacks under internal locks.

Avoid global mutable operation state, process-wide configuration changes, implicit CPU affinity and unbounded caches. Large-machine optimisation may add measured NUMA or affinity policies later, through optional host configuration. It must not become a prerequisite for small-machine correctness.

## 10. Portability, Rust baseline and acceleration

Name supported CPU/OS/ABI/toolchain profiles. The primary release matrix covers Windows x64, Windows ARM64, Linux x64, Linux ARM64 and the selected Android library/binding profile. Android is a release qualification target from the beginning. The recommended first Android device ABI is arm64-v8a (Rust aarch64-linux-android), with x86_64 (Rust x86_64-linux-android) qualified separately for emulator/CI use. Physical-device execution is required for the selected Android device ABI; for the recommended arm64-v8a profile, use physical ARM64 hardware. Emulator evidence does not replace device execution. macOS, particularly Apple Silicon, is a secondary compatibility target. Preserve portable design and practical build checks without making Apple integration a gate for the initial primary-platform release. A selected RV64 Linux profile, other Apple variants, 32-bit processors and bare-metal targets enter through separately scoped consumers. Rust compiler support tiers and the programme's executed qualification remain different records. [S07, S42]

| First release profile | Required agreement across the delivered stack |
| --- | --- |
| Windows x64 | x64 JVM, native launcher, Rust libraries and any optional renderer adapters |
| Windows ARM64 | Native ARM64 JVM, launcher, Rust libraries and optional adapters; x64 emulation is not native ARM64 qualification |
| Linux x64 | x64 JVM and Rust libraries on named distribution/libc and display profiles |
| Linux ARM64 | ARM64 JVM and Rust libraries on named distribution/libc and display profiles |
| Android device profile (recommended arm64-v8a) | Kotlin facade and JNI bridge running on Android, matching Android-target Rust libraries, AAR/consumer packaging, named API/device profiles and physical-device execution |
| Android emulator/CI profile (recommended x86_64) | Matching Android native libraries for the selected emulator ABI (recommended x86_64) and an instrumented consumer; report emulator results separately from device qualification |

For the recommended Java 25 baseline, Microsoft lists native binaries and certified architectures for all four desktop combinations. This establishes a runtime candidate, not executed DICOM Capacitor compatibility. Pin the selected build and hashes, name OS versions and Linux distributions, and run the actual packaged application on each target. Microsoft currently lists September 2030 as the earliest end of support for its Java 25 LTS line; desktop use outside Azure should not be described as covered by its Azure commercial-support offer. [S29]

Embedded Linux does not automatically require no_std. Keep algorithmic foundations separable from I/O and scheduling so a justified no_std/alloc profile can be introduced for a real consumer. Bare-metal RAM, allocator and atomic limitations must be specified before promising such a profile.

Select and pin a stable Rust toolchain at bootstrap. Record the edition and minimum supported Rust version in the manifests, and test the promised compatibility range. Update the toolchain deliberately and record its effect on correctness and performance. Nightly-only tooling may support development tests; the portable release baseline must not acquire a hidden nightly requirement. Cargo's rust-version field records a package's toolchain support contract. [S08]

Use explicit endian conversion for encoded data, checked integer-width conversions and correct alignment handling. Do not select target instructions by detecting the build host. Keep a portable CPU correctness path and enable SIMD only where the target and operating environment support it. A fallback must not execute unsupported instructions.

Optimisations must preserve exact lossless samples. Resampling and lossy operations need predefined numerical criteria, with scalar and accelerated paths tested against them. No silent precision reduction, approximate transform or fast-math change is acceptable.

SIMD and GPU work follows profiling. Measure complete operations, including allocation, transfer, synchronisation and output handling. Unified physical memory does not remove ownership or synchronisation duties. Optional hardware-native paths must report the backend actually used and their fallback policy. A forced unsupported backend returns a clear error.

Platform-specific bindings, supported numerical primitives and drivers are explicit, optional adapters. They must preserve the core contracts and must not substitute an undeclared external DICOM or compression engine. Apple-specific acceleration in SIS is a design reference, not evidence of equivalent execution on another platform. Each operation and target earns its own correctness and performance claim.

### 10.1 Hardware and OS-specific backends

Follow DICOMkit/SIS's philosophy of using the host effectively. Keep the portable CPU implementation useful, then add CPU SIMD, OS numerical facilities and device kernels where they improve an admitted workload. Compile optional backends separately; provide explicit selection and disabling for comparison. Runtime checks must verify the actual instruction set, device features and OS/driver capabilities.

| Hardware family | Mechanisms to evaluate and qualify |
| --- | --- |
| Apple Silicon | ARM CPU SIMD, Accelerate/vDSP/vImage, Metal compute and explicitly shared resources on UMA systems; use supported public interfaces [S16] |
| NVIDIA | CUDA kernels and suitable numerical primitives, streams, explicit transfers and Unified Memory where appropriate; graphics may use the qualified wgpu backend [S17, S21] |
| Qualcomm Snapdragon | CPU SIMD, Adreno Vulkan/OpenCL paths on supported stacks, and suitable Hexagon/HVX or supported NPU mechanisms where an operation fits; qualify Android and Windows separately [S18] |
| Intel Lunar Lake / Core Ultra 200V series | Supported CPU SIMD and GPU paths through qualified Vulkan, Level Zero, SYCL or OpenCL configurations; evaluate supported NPU operations separately [S19, S21] |
| AMD Strix Point and Strix Halo, including Ryzen AI Max+ 395 | Supported CPU SIMD, qualified Vulkan or HIP/ROCm GPU paths, and separately justified Ryzen AI/XDNA workloads; check exact SKU and OS support [S20, S21] |
| Other ARM and RISC-V targets | Portable CPU path first; optional supported vector extensions or devices only through measured, capability-checked profiles |

These are backend candidates and target families, not a promise that every combination is available or required for the first release. Record the exact operation, type/precision, layout, device, OS, driver, SDK and deployment/licence conditions. A GPU support entry is not an NPU support entry. NPU or matrix acceleration is useful only when the required operation maps correctly and benefits measurably; do not insert quantisation, neural inference or reduced precision merely to use available hardware.

Treat UMA, shared virtual addressing and managed memory as different mechanisms. A common address or physically shared RAM does not prove a copy-free pipeline. Record owners, compatible allocation/texture layouts, access transitions, fences and completion. Account for staging, migration, residency and retained storage as well as explicit copies. CUDA's Unified Memory behaviours differ by platform; other vendors' cross-API sharing likewise requires a qualified memory and synchronisation contract. [S16, S17, S18]

For every accelerated route, compare required results against justified expectations and the portable path; measure complete costs, including transfers, setup, synchronisation and failure/cancellation. Automatic fallback must follow a documented policy and report the backend used. A caller requiring a specific unavailable backend receives a clear unsupported result. Bring backends into the release incrementally rather than making all vendor integrations prerequisites for useful delivery.

## 11. Foreign interfaces and host integration

Provide an idiomatic Rust API and a deliberately versioned C ABI for foreign consumers. Rust's native ABI has no stability guarantee; Rust can produce system-library outputs appropriate for other languages. [S12]

Use opaque handles, fixed-width fields where appropriate, explicit lengths, defined string encoding, versioned structure layouts and capability negotiation. Define calling conventions and conversions for each target. Rust Vec, String, references, trait objects and futures do not cross the C ABI. repr(C) is a layout mechanism, not a portable file format.

The caller retains explicit pointer-validity obligations. Checking a length does not make an arbitrary dangling foreign pointer safe. Bindings must preserve lifetimes and allocator ownership and batch useful work instead of making per-sample foreign calls. Keep handles from different engine instances or ABI generations distinct.

No Rust panic or foreign exception may escape an ordinary ABI boundary. Expected errors must use normal result/status paths. If panic containment is used, recover only to a documented safe state. catch_unwind catches unwinding panics; it does not catch aborting panics or provide process isolation. [S13]

Primary host integration comprises Java for desktop DICOM Capacitor and Kotlin/JNI for Android. The desktop work order selects one FFM or JNI route for its pinned JDK baseline; section 11.2 recommends FFM with Java 25. Android has a distinct JNI/NDK route under section 11.4 and shares the Rust implementation and ownership/error contracts. Later profiles include C# native interoperation and Swift wrappers over the documented C interface. Keep lifecycle and application scheduling with the host where practical. Package, load and update tests belong to the binding's acceptance evidence; compiling a shared library alone is insufficient.

Browser WebAssembly is a separately qualified profile. Server-side use underneath Jakarta EE/TomEE does not require a browser rewrite. Database, archive policy, application UI and physical media-device control remain host responsibilities unless separately assigned. DICOMflux-3D and the Apple Voxelia adapter provide the admitted presentation capabilities; applications retain their workflow and lifecycle responsibilities.

### 11.1 Swift bindings and compile-time comparison

Swift is a supported binding language for DICOMflux and each applicable Flux Imaging Suite module. Provide a thin Swift facade that mirrors the verified public names, operations and semantics of DICOMkit or the corresponding SIS library. Pin the reference version and maintain a tested API inventory, including defaults, errors, precision, ownership, cancellation and concurrency. Claim substitutability only for the covered surface; missing capabilities must be reported explicitly.

Select the core at build time. The normal Apple application configuration continues to use DICOMkit/SIS. An explicit comparison configuration selects Rust through the Swift facade, with covered application call sites unchanged apart from documented package/build selection. Keep backend implementation modules distinct and expose one selected application-facing surface. Do not silently call the Swift engine to fill gaps while reporting Rust execution.

The package/target graph, compilation conditions and native link inputs must agree. A conditional import or #if alone is insufficient evidence that the unselected core is absent from the binary. Test the two ordinary configurations separately; isolate any deliberate dual-engine comparison harness. Backend identity and the implemented capability profile must be visible in reports. SwiftPM provides target/product/build configuration mechanisms, while the exact facade and selection scheme are project contracts. [S15]

Use the versioned C ABI and module interface beneath idiomatic Swift wrappers. Package qualified Apple binary variants through XCFrameworks where appropriate, with the required device and simulator slices. Verify the header, ABI, native library, wrapper and Swift package as one deliverable. Follow the assigned DICOMkit/SIS Swift version, strict-concurrency rules and OS floors; do not claim an Apple platform or variant merely because the Swift wrapper compiles. [S15]

Pointers obtained through scoped Swift buffer APIs MUST NOT outlive the documented access scope. Retaining the Swift Array, Data or wrapper object does not by itself extend that pointer's validity. Asynchronous Rust/device work MUST use storage whose address and permitted access remain valid until actual completion, through a documented stable allocation and ownership contract or an explicitly permitted copy. A synchronous borrow is valid only if all work using it completes before the scope ends. Wrapper deinitialisation and Swift task cancellation cannot release in-flight storage. Sendable claims, callback executor rules and actor boundaries require demonstrated semantics; avoid unchecked declarations that merely suppress diagnostics.

Run the same covered API examples, fixtures, algorithm profiles and resource limits against both build configurations. Compare meaningful results, errors and preservation guarantees, then measure core time and complete Swift-to-core operation costs. Record compiler, selected implementation, accelerator, workers, conversions and allocations. Agreement supplements independent correctness evidence. A successful comparison does not automatically change the default Apple core or replace Voxelia.

### 11.2 Recommended desktop Java baseline and Rust binding

For desktop DICOM Capacitor, qualify Java 25 LTS with Swing and AWT from the JDK's java.desktop module. Microsoft Build of OpenJDK is the initial distribution candidate because it supplies the four required desktop OS/architecture combinations. Pin a maintained servicing update and its hashes in the implementation work order. A different distribution requires checking the same native targets, desktop components, licence, update policy and actual execution. [S29, S31]

For desktop Java, prefer the standard Foreign Function and Memory API for the versioned Rust C ABI. FFM became standard in Java 22; Java 25 provides it in java.lang.foreign without an incubator dependency. Keep the Java application behind a small idiomatic facade so binding implementation details do not spread through Swing code. A full desktop JNI binding is a contingency for an evidenced need. Android's required JNI integration is a separate target profile and does not depend on the desktop FFM choice. A separate JAWT graphics adapter may legitimately require JNI. [S30, S34]

The Java facade MUST provide deterministic ownership, typed errors, capability and ABI-version checks, explicit string encoding and bounded buffer descriptors. Use AutoCloseable owners and documented release functions. Do not depend on garbage collection or finalisation as the normal release mechanism for datasets, buffers, operations or GPU resources. Rust allocations are released through the Rust contract, not an unrelated Java allocator.

For the initial binding, run synchronous Rust operations on bounded Java background workers. A call may perform bounded Rust parallel work, but its borrowed storage remains valid until all work using that borrow has completed. Separate operation-scoped memory from long-lived dataset and buffer leases. If native asynchronous operations are later added, retain every needed handle, arena, library lookup and callback stub until native completion is acknowledged.

FFM arenas enforce Java-side lifetime and access rules; they do not make arbitrary native pointers, incorrect function descriptors or Rust code automatically safe. Confined arenas belong to their creating Java thread; shared arenas permit access from multiple Java threads but do not supply application-level synchronisation. Use the actual C ABI layouts for each target and verify fixed-width fields, pointer sizes, alignment and structure padding. [S30, S32]

Native callbacks MUST NOT let Java exceptions escape or directly update Swing widgets. Translate failures, retain callback storage and marshal bounded notifications through the application. Use coarse operations such as inspect-dataset, decode-frame, prepare-presentation and resample-region rather than one foreign call per pixel or per small metadata field.

Package the native-access permission for the binding module. A class-path-only packaging profile may explicitly use ALL-UNNAMED; prefer a named or stable automatic binding module when practical. Include a qualification run with illegal native access denied. Do not rely on the current default warning to hide missing launcher configuration. The launcher options and native library lookup paths are part of the tested package. [S30]

### 11.3 Swing application and presentation contract

Swing owns controls, navigation, application state and interaction. DICOM parsing, supported codec work, quantitative transforms and advertised resampling remain in the Rust libraries. The DICOMflux image adapter interprets DICOM presentation metadata and composes the appropriate generic imaging operations. Java2D may draw completed presentation images and interface graphics; it does not become a second DICOM codec implementation.

Keep parsing, decoding, resampling, volume preparation and GPU waits off Swing's Event Dispatch Thread (EDT). Create and update Swing controls and attached UI models on the EDT. A bounded executor with EDT completion dispatch is the initial recommendation; SwingWorker may be used with an explicit surrounding resource policy. Its default scheduling must not be treated as the programme's worker or memory budget. [S33]

Batch metadata updates, bound pending jobs and coalesce rapid window/level or frame-selection requests. Associate results with a request generation so a stale completion cannot replace the current image. Java interruption or cancellation is a request: connect it to the native cancellation mechanism, retain resources until native work has actually ended, and never wait for that acknowledgement on the EDT.

Preserve original typed samples, explicitly transformed quantitative values and display pixels as different representations. Keep source signedness, meaningful bits, units and geometry. Do not treat a packed Java int colour pixel as a 32-bit numerical image sample, or convert arbitrary 32-bit integer samples to float while claiming exact preservation. Pixel inspection, measurements and lossless output use the appropriate original or quantitative representation.

For the initial viewport, use a bounded, explicitly laid-out presentation buffer copied into a completed BufferedImage when required. A worker prepares an exclusive back buffer, then the EDT publishes the completed front image. Painting draws completed output and never starts a decode or races a worker rewriting the displayed storage. Record copies and allocations rather than describing this baseline as zero-copy.

BufferedImage supports unsigned-short greyscale, but getRGB() converts to default sRGB with eight bits per component. Display colour conversion and quantitative data access are distinct contracts. A CPU presentation path does not establish end-to-end HDR, ten-bit display or diagnostic display calibration. Such features require their own specified pipeline and verification. [S36]

Use Mayam's workflow and interaction experience to reduce design work. Keep the initial UI compact: open/select input, metadata tree and search, an admitted 2D viewport, operation controls, capability reporting and results. Inspect the actual DICOM Studio scope before claiming equivalence; the first demonstrator is a selected subset.

### 11.4 Android Kotlin and JNI integration

Android applications use an idiomatic Kotlin facade over a thin JNI bridge to the same Rust implementation. Build native libraries for Android with a pinned compatible Rust/NDK toolchain and Android target sysroot. Android's runtime and available Java APIs are distinct from the desktop JDK used to run build tools. The desktop Java 25, Swing, FFM and jpackage recommendations do not define the Android application runtime or binding. Keep JNI glue small and preserve the common native ownership, error, capability and cancellation contracts. [S41, S42]

Make useful coarse-grained calls from bounded Kotlin background workers. Parsing, codecs, resampling and waits for native completion MUST NOT block the Android UI thread. Coordinate host scheduling with Rust worker limits so their combined concurrency remains bounded. Completion returns through the host's chosen lifecycle-aware dispatcher. A cancelled coroutine, Activity recreation or closed screen does not establish that native work has stopped: retain the affected handles and buffers until completion is acknowledged, suppress stale results and define who owns any continuing task.

JNIEnv is thread-local and MUST NOT be shared across threads. Attach native-created threads before they call JNI and detach them when required. Use appropriate local/global references for their actual scope, release them predictably and handle pending Java exceptions before further JNI work. A Rust panic MUST NOT unwind through JNI. Map expected native failures to documented Kotlin results or exceptions, preserving the native error category. [S41]

Prefer caller-owned buffers or explicit copies until a native-backed view has a verified lifetime protocol. A direct ByteBuffer is not an ownership transfer: keeping its managed wrapper alive does not by itself preserve a Rust allocation. Specify lengths, offsets, strides, sample precision, signedness, byte order, alignment and whether the contents may change. Explicit close/release must coordinate with in-flight work; cleaners or finalisation cannot be the normal resource policy. Original samples and derived display pixels retain the distinction required by sections 6 and 7.

Keep Android storage access with the host. A content URI is not a native filesystem path. Provide bounded input/output adapters using owned buffers, streams or descriptors with explicit lifetime, offset, length and seeking contracts. Content providers may supply non-seekable streams; where an operation requires random access, use a declared bounded staging policy or return an unsupported-input error. Do not stage an unbounded study merely to satisfy a path-only interface. The host owns permission handling and output publication. [S45]

Use realistic device budgets for memory, queued frames, workers, thermal behaviour and sustained battery cost. Preserve a useful CPU path; device-specific SIMD or Qualcomm and other vendor acceleration remains optional and separately measured. Oviyam | Mobile creates a separately qualified Android 3D rendering requirement. Reuse of the Windows/Linux wgpu component is a candidate for investigation, not an established Android deliverable. A minimal Kotlin consumer/instrumentation harness proves the Android library contract without requiring an Android DICOM Capacitor product. The Android DICOM Compressor application remains independently scoped.

## 12. Verification and independent evidence

Every requirement needs a justified observable expectation. Use standards-derived cases, small hand-checkable examples, permitted conformance material, independent generators and external comparison implementations. Self-round trips and Swift/Rust agreement are valuable supplementary checks; related implementations can share a mistake.

| Evidence group | Required examples where applicable |
| --- | --- |
| Standards and valid boundaries | Minimal and unusual valid encodings, precision/sign limits, sequence/frame structures and exact format limits |
| Malformed and adversarial data | Truncation, inconsistent lengths, overflow, invalid offsets, deep nesting, decompression/work amplification and excessive fragments |
| Memory and ownership | Strides, offsets, aliasing rejection, read/write lifetimes, partial failure, cancellation, concurrent readers, dropped futures, aborted tasks, wrapper disposal and no reuse before worker/hardware completion |
| Interoperability | Our encoder to an independent decoder and independent encoder to our decoder; declared DICOM integration checked separately |
| Fidelity | Exact lossless samples, defined lossy tolerances, preserved metadata meaning and required bitstream/object guarantees |
| Concurrency | Worker counts, nested scheduling, races, deadlocks, callback reentrancy and cancellation at meaningful stages |
| Portability and delivery | Target execution, feature subsets, independent consumers, binding packages and loading behaviour; both Swift core selections and native link/dependency closure |
| Acceleration | Scalar/SIMD/GPU equivalence for the same corpus, explicit backend selection, unsupported devices, device loss and bounded GPU work |
| 3D and presentation | Volume geometry, coordinate/intensity conventions, quantitative-source preservation, renderer/UI separation, capability limits and the assigned Voxelia interoperability path |

Use property testing, fuzzing, targeted unsafe/lifetime analysis and supported sanitizers according to the risk. Record tools, budgets, seed/corpus identities and results. Missing hardware, a missing oracle or an unsupported analysis tool is missing evidence, never a pass. Confirm that tests can detect selected realistic faults where it materially increases confidence; avoid test-count targets and tests that merely restate the implementation.

Keep normal development and regression fixtures locally available. The internally versioned corpus must increment when its contents or expected results change, with fixture hashes, permissions, generators and provenance. Hosted runs may retrieve the same pinned corpus from authorised storage and verify it. Ordinary tests must not depend on unpinned public downloads.

Preserve failing examples and explain discrepancies. A standard defect, an unsupported profile and an implementation defect need distinct dispositions. Do not change an oracle expectation just to make the implementation pass. When a previous implementation was wrong, correct the contract or implementation with traceable evidence rather than preserving the defect as compatibility.

## 13. Performance and resource acceptance

Each work order must set measurable budgets before a capability is declared complete: peak live memory, scratch/output bounds, worker and queue limits, maximum admitted workload and any latency, throughput or binary-footprint targets. Choose numbers from the target device and use case. There is no universal acceptable memory ceiling or speed-up percentage.

Measure correctness-qualified code with reproducible workloads. Record CPU/device, OS, compiler, dependency and source revisions, build flags, backend, thread count, image descriptors, dataset hash, warm/cold conditions and repetition/variance. Distinguish allocated bytes, resident memory, scratch estimates, GPU memory and queue occupancy instead of treating them as interchangeable measurements.

Report end-to-end latency and throughput as well as selected hot loops. Include failure/cancellation costs, compressed-size changes where relevant and the small-device profile. Do not exclude managed/native copies or GPU transfers from a claim about application-level performance.

Record deterministic behaviour. Exact sample preservation is mandatory for lossless paths. Repeated independent encodes of the same samples require byte-identical output only where the named implementation/profile promises deterministic encoding. This does not relax a declared compressed-bitstream or whole-object preservation guarantee: those operations must recover the original promised bytes. A new encoder strategy may change valid packetisation only where no original-byte preservation or applicable deterministic-output contract is violated. Thread-count and backend effects must be stated and tested.

Once a baseline exists, define meaningful regression thresholds and noise treatment. A confirmed regression requires an explanation and disposition, with security/correctness trade-offs made visible. Do not improve a benchmark by weakening validation, limiting the test corpus silently or changing fidelity. A faster unsafe path must earn its place against the safe baseline.

## 14. CLI, packaging and maintainability

Mirror the assigned SIS CLI vocabulary and meanings. Keep common operations such as inspect, encode, decode, validate and capabilities consistent where they are supported; the executable name reflects the Flux module. A reserved or unimplemented operation must fail honestly before payload work.

Retain coherent input/output selection, format/profile selection, worker/resource limits, cancellation/timeouts, backend/copy policy, overwrite handling and machine-readable diagnostics. Keep stdout payload/result streams separate from bounded stderr diagnostics. Version the machine-readable output schema. Exact aliases and exit-status mappings come from the verified common CLI contract.

File-producing tools need explicit overwrite and partial-output policies, safe temporary locations and atomic publication where the host supports it. Cancellation must not leave a partial output presented as a completed result. CLI operations must call the same library implementation used by applications, not a duplicate algorithm.

Keep crate APIs small and documented with working examples. Test a consumer outside the workspace to catch private path dependencies and accidental feature coupling. Minimal and documented feature combinations must build; optional features must not unexpectedly add a codec, GUI, network runtime or licence obligation to the minimal profile.

Version public API, foreign ABI, shared contract, toolchain support and capability metadata deliberately. Use semantic versioning with migration notes, including explicit treatment of 0.x changes. Pin dependency resolution for release evidence and CI while also checking the published library's declared dependency and compiler compatibility.

Releases need source identity, enabled capabilities, dependency inventory, notices, reproducible build instructions, test/performance evidence and SHA-256 integrity files. Reproducible instructions do not imply byte-reproducible binaries; make the stronger claim only after demonstrating it. Digests establish integrity relative to a manifest, not publisher authenticity; retain applicable signing and provenance information separately.

### 14.1 Documentation character and optional synthetic demonstration

Human-readable documentation and help MAY contain restrained references to Back to the Future, the Flux Capacitor and 1.21 gigawatts. Keep the functional explanation first and the joke occasional. For example, a demo help footer may say "No 1.21 gigawatts required." Such text must not replace error explanations, alter exit statuses or unexpectedly enter machine-readable output, logs containing patient information or ordinary patient metadata.

An optional explicit CLI demonstration may generate an original procedural Flux Capacitor illustration and write it as DICOM using the requested supported codec/profile. This is a candidate optional feature, implemented only when included in a scoped work order; this document does not claim that it exists. The exact command or flag must fit the common CLI contract. One illustrative form is:

```text
dicomflux demo flux-capacitor --codec fluxj2k --profile lossless --output flux-capacitor.dcm
```

Ordinary commands must not trigger the demonstration. Keep generation local, bounded, versioned and opt-in; it should not require a GPU, a network service or a large asset dependency. Use project-owned drawing code or assets and make the image visibly recognisable as a synthetic demonstration. The demonstration MUST use exclusively synthetic pixels and metadata. It MUST NOT read or incorporate patient images, identifiers or clinical datasets. The procedural renderer produces a generic image buffer; DICOM construction remains in the DICOMflux CLI/integration layer.

Start with one explicitly qualified single-frame Secondary Capture profile. Complete all applicable required attributes and emit a valid Part 10 file with consistent SOP identifiers, File Meta Information and Pixel Data. Set Synthetic Data (0008,001C) to YES and the applicable SC Conversion Type (0008,0064) to SYN. These are standard attributes and values. [S24]

Use unmistakably synthetic identity and descriptive fields. A default may use Patient's Name SYNTHETIC^FLUXCAPACITOR and Patient ID DICOMFLUX-DEMO; the optional fictional-character presets below may substitute names while retaining explicit synthetic identifiers, labels and standard synthetic-data attributes. Study and series descriptions must identify the demonstration. Creation timestamps and implementation identity describe the real generator. Generate valid, distinct UIDs under the established policy; never encode jokes as UID roots, anatomical measurements or fabricated source-instance references. [S25]

The owner-requested fictional name set includes Marty McFly, Lorraine Baines, Doc Brown and Dr. Emmett Brown. Encode person names using DICOM PN component rules: examples are McFly^Marty, Baines^Lorraine and Brown^Emmett^^Dr. Preserve Doc Brown as a clearly fictional alias in an appropriate comment or preset label. Each preset has a synthetic Patient ID and visible demonstration labelling; it never changes ordinary patient metadata. [S37]

The optional quotation set supplied by the owner is:

- “Roads? Where we’re going, we don’t need roads”

- “Great Scott!”

- “If you put your mind to it, you can accomplish anything”

- “If my calculations are correct, when this baby hits 88 miles per hour, you’re gonna see some serious shit”

Keep the longer quotations in a suitable comment/text attribute admitted by the selected IOD, with its VR, length and character set validated. They must not be forced into short description fields or silently truncated. Where the generator retains curly apostrophes or other non-default characters, use a supported declared character set. The full quotation set, including its explicit language, belongs only to the deliberately selected synthetic demonstration; ordinary help remains restrained. Film references and fictional dates are descriptive content, never fabricated acquisition facts. [S37]

Preflight the selected codec, encode profile, sample precision, components, photometric interpretation and IOD together. Use the registered Transfer Syntax for the actual output; reject an unavailable or incompatible choice before publishing a file. There is no silent codec substitution. DICOM RLE uses the DICOM frame profile, not bare PackBits. Reversible reconstruction of an existing JPEG bitstream is a different operation from direct encoding of newly rendered pixels. Explicit lossy modes must record their actual lossy status, method, ratio and applicable derivation information. [S10, S25]

A pixel seed fixes the procedural image, not normal instance identity. Ordinary runs create fresh identifiers; a fully pinned regression fixture may deliberately reproduce its own identity and timestamps. Validate the resulting object and independently decode its pixels where available. This demonstration exercises the public writer/codec path and supplements the substantive conformance and fidelity corpus.

### 14.2 Desktop Java application packaging

The recommended packaging path is a bundled application image built with jlink/jpackage, containing java.desktop and the other actually required modules, the Java application and matching Rust libraries. Users should not need to install a separate JDK for that distribution. Record service-provider modules and resources required by the application so runtime minimisation cannot remove a necessary component. [S38]

Build and execute one package profile for each initial desktop OS/architecture combination. jpackage requires packaging on the target platform; do not assume a single host produces every native installer. Start by qualifying the bundled app-image and launcher, then add the selected installer formats and their tooling. Check those tools' actual licences as well as runtime redistribution. [S38]

For the initial demonstrator, distribute an archive of the qualified bundled application image where that meets the work order. Windows MSI/EXE generation through jpackage requires suitable WiX tooling; current WiX releases have a separate maintenance-fee/EULA policy for covered commercial use. Do not assume that route has no payment condition, or select an obsolete tool release merely to avoid reviewing the current terms. Keep the later installer choice separate and qualify its maintenance status, licence and ARM64 behaviour. [S38, S39]

The first Windows ARM64 packaging check must open a Swing window and execute a real Rust operation using a native ARM64 JVM and libraries. Perform equivalent packaged execution on Windows x64, Linux x64 and Linux ARM64. Name supported Linux distributions, libc baseline and display environments; a glibc package does not imply musl support.

Load native libraries from an explicit controlled application location, check ABI/capability compatibility and report architecture mismatches clearly. Include source/build identities, runtime and dependency notices, source-availability material where required, hashes and an update procedure. The bundled-runtime release process must ship relevant JDK security updates; an installed system Java update does not automatically replace the application's private runtime. [S29, S38]

### 14.3 Android packaging and qualification

Deliver the Kotlin facade, JNI bridge and matching native shared libraries as an Android Archive (AAR), with declared dependencies and any required consumer shrinker rules. Place each native library under its correct ABI. Qualify loading and ABI/capability negotiation in a separate consuming application, including its release build and shrinking configuration where used. Android builds use their own target/sysroot; a desktop Linux shared library is not an Android deliverable. [S42, S43]

Record the supported ABIs, minimum Android API, compile/target API levels, Rust, NDK, Android Gradle Plugin, Gradle, build JDK and Kotlin versions in the work order. Choose the minimum API from actual consumer and device requirements; this manifesto does not freeze it. Keep those decisions separate from the recommended desktop Java baseline.

Require compatibility with both 4 KB and 16 KB page-size environments. Inspect every shipped native library, including Rust-produced shared objects and native dependencies, for the applicable ELF load-segment and RELRO layout requirements. Verify native-library alignment in the final consuming APK as well as the library outputs. Avoid hard-coded page-size assumptions and execute the packaged consumer in both environments without relying on compatibility mode. Selecting a recent NDK alone does not qualify a custom Cargo/linker/packaging pipeline. [S44]

Run the declared read/write, decode/encode and resampling capabilities through Kotlin/JNI, as each is implemented, with resource limits, repeated load/release, cancellation, lifecycle changes and buffer-lifetime checks. Record physical-device execution for the selected device ABI (recommended ARM64) and emulator evidence separately. The Android application exclusion does not waive library, binding or consumer-package qualification. Deliver source/build identities, supported-profile records, notices, documentation and SHA-256 integrity material.

## 15. Coding-agent work protocol

### 15.1 Start with a bounded assignment

Read the repository's instructions and working state, preserve unrelated changes and identify the authorised branch or workspace. The module work order must name the intended capability, exclusions, source baselines, target profiles and acceptance criteria.

Do the useful implementation and verification already authorised. Do not turn a small module assignment into a platform rewrite, new umbrella architecture or unrelated refactor. Do not restart accepted logical design merely to produce another document. Create only the records needed to make the work executable and reviewable.

If a required source, normative clause, ownership rule or acceptance criterion is missing, identify the exact gap and continue independent work where possible. Do not invent it. Public API changes, shared contract changes, new dependencies or expanded capability claims must receive the applicable project review rather than entering silently.

### 15.2 Minimum work-order record

The following fields can live in one concise work-order file; separate documents are unnecessary unless the assignment warrants them.

| Field | Required content |
| --- | --- |
| Assignment | Module, bounded outcome, exclusions and authority for repository actions |
| Sources | DICOM/codec clauses, document editions/digests, approved DICOMkit/SIS or admitted Mayam commits, reference-use licences and applicable contribution-assignment records |
| API | Swift-to-Rust equivalence, Swift facade coverage/build selection where assigned, desktop Java facade/binding route, Android Kotlin/JNI coverage where assigned, public 3D boundaries, ownership/errors and compatibility impact |
| Capability | Exact encode/decode/read/write/service profiles and expected unsupported cases |
| Targets | CPU/OS/ABI, Rust and applicable desktop Java, Android API/NDK/Gradle/Kotlin or Swift toolchain contracts; features, selected core/backend, SDK/driver, page-size profiles and required real-device execution |
| Budgets | Input/output/work/memory/worker/queue limits and benchmark acceptance |
| Evidence | Fixtures, independent expectations/oracles, negative cases, concurrency and packaging checks |
| Completion | Source identity, commands/results, deviations, limitations and next bounded step |

### 15.3 Implement and report honestly

Develop small vertical slices: validated input, useful operation, explicit output, failure handling and independent verification. Prefer one working capability exercised by real consumers over broad scaffolding with success stubs.

Keep changes reviewable. Every unsafe block and FFI contract needs a clear justification. Tests must exercise actual code paths and backends. An external oracle may support a test, but cannot become a shipped implementation shortcut. Update capability reporting as behaviour is implemented and qualified.

Completion reports must separate **implemented**, **compiled**, **executed**, **verified against stated criteria**, **not run** and **blocked**. Include exact commands, source/build identities, target configurations, fixture hashes, results, resource measurements and unresolved limitations. Never report a planned test as executed or an emulated run as a physical-device result.

Keep source and evidence continuity when handing work to another agent. Record open decisions and the next executable step, not a replacement history. The same requirements should survive a change of coding agent or programming environment.

## 16. Definition of done and delivery order

A scoped capability is done only when:

- Its project-owned implementation follows the authorship rules, and admitted dependencies or reused components have documented provenance and qualification evidence.

- Standards, supported profiles and unsupported cases are recorded.

- API equivalence and intentional Rust differences are documented.

- Ownership, lifetimes, thread-safety, cancellation and partial-output contracts are exercised.

- Relevant malformed-input and resource-exhaustion cases behave predictably.

- Independent expectations establish the promised fidelity and interoperability.

- Required targets and feature profiles have the declared execution evidence.

- Resource/performance budgets and any regressions have a recorded disposition.

- Public examples, CLI/bindings where assigned, capabilities and notices match the implementation.

- Reviewable source, results and SHA-256 integrity records are delivered without overstated claims.

Completion of a module does not automatically qualify an application, display pipeline, archive service or clinical workflow. Those integrations use the module's evidence and add the evidence needed for their own claims.

### 16.1 Delivery sequence and DICOM Capacitor

DICOM Capacitor is the programme's DICOM Studio-equivalent showcase application. Use it as a real consumer of public APIs and a visible demonstration of toolkit progress. Inspect the relevant DICOM Studio capabilities and select an explicit equivalence inventory; the name does not by itself commission a complete workstation. Do not duplicate parsers, codecs, resamplers or renderers inside the application.

| Milestone | Useful outcome and completion basis |
| --- | --- |
| 1. Rust contracts and foundation | Pin source/API baselines; implement minimal useful Rust modules and versioned C ABI with portable correctness, ownership and resource budgets. Qualify native loading on the four desktop targets and the selected Android profile. |
| 2. First consumers and bindings | Deliver bounded local read/inspect/decode functionality through the Java/Swing demonstrator and an early Kotlin/JNI consumer harness. Keep Swing UI work and Android instrumentation as separate consumers of the same Rust contracts, with responsive cancellation and explicit unsupported cases. |
| 3. Imaging operations and output | Add the assigned independent codec/resampling profiles and writer/transcode paths. Expose declared capabilities through CLI, desktop Java and Android Kotlin interfaces, with fidelity/resource reports and safe output handling. |
| 4. Initial primary-platform release | Package and run the selected useful subset on Windows/Linux x64 and ARM64; deliver and qualify the Android AAR/native profile through its consumer harness. Deliver source, notices, contribution records, documentation, results and SHA-256 integrity material. No Android DICOM Capacitor application is required. DICOM Compressor's Android application scope is a separate decision. |
| 5. Discrete 3D integration | Qualify the bounded Windows/Linux wgpu renderer and optional egui harness independently, then qualify offscreen output or the selected Swing native viewport adapter. This is a separate capability milestone. |
| 6. Acceleration, additional bindings and compatibility | Add measured vendor backends and scoped C#/Swift integrations. Complete the covered Swift comparison configurations and relevant Voxelia adapter without delaying the initial primary-platform release. Android's primary Kotlin/JNI binding is already part of milestones 1-4. |
| 7. Showcase expansion | Extend the selected DICOM Studio-equivalence inventory and toolkit services according to demonstrated needs. Include optional synthetic demonstration features when assigned and verified. |

This is a dependency-based sequence, not a set of invented calendar dates. A showcase slice can begin as soon as its required public APIs work; it must not wait for the entire codec portfolio. Independent modules and backend investigations may proceed in parallel once shared contracts are fixed. Each milestone has bounded acceptance criteria and no requirement to finish every encoder, decoder or vendor simultaneously.

For Windows and Linux, the initial DICOM Capacitor showcase uses Java/Swing. Android's primary delivery is the Rust libraries, Kotlin/JNI integration and a small validation consumer; an Android DICOM Capacitor application is not required. Android DICOM Compressor remains independently scoped. The Rust core and imaging modules have no dependency on Java, Kotlin, Swing, a JVM, Android UI, egui or a window system. Optional native 3D adapters and later language bindings consume the same public contracts. Apple comparison builds retain the requested Swift facade and Voxelia direction on their secondary schedule.

Where two authorised consumers are available, exercise a shared capability through both before broadening its abstraction substantially. Otherwise keep the abstraction minimal and record the untested consumer assumptions. This guidance does not require product integrations outside the work order or prevent delivery of an independently verified standalone module.

### 16.2 Recommended first capability profile

Use a bounded local-file profile to prove the core and binding before expanding application workflow. The first work order should name supported Part 10 encodings, SOP Classes and image representations explicitly. A suitable candidate is Explicit VR Little Endian with an admitted single-frame greyscale profile covering unsigned eight-bit samples and signed/unsigned samples in sixteen-bit storage. Add other encodings and multiframe/volume arrangements through separate capabilities, with truthful inspection-only behaviour where pixel decoding is unavailable.

The first desktop DICOM Capacitor application should open a file, search the Rust-exposed metadata, display a supported frame, adjust window/level, pan/zoom, inspect original and transformed values, report the selected backend/capabilities, and cancel or close safely. Keep underlying samples unchanged. Resampling comparisons and output operations must identify the actual Flux kernel and profile rather than presenting an ordinary UI resize as toolkit verification.

A recommended first compressed path is FluxRLE's DICOM frame profile, followed by an independently qualified FluxJLS lossless profile. This is a delivery recommendation, not a reduction of the FluxJ2K/HTJ2K, FluxJXL, FluxJLI or seven-kernel FluxResample portfolio. Parallel codec work can proceed after the shared buffer, error, limit and evidence contracts are stable. Do not delay an independently useful module while waiting for every codec.

The first resampling slice can establish nearest-neighbour and bilinear semantics, then extend the assigned seven-kernel portfolio with its antialiasing and boundary rules. Add compressed writing/transcoding only after the relevant writer and codec profiles are both qualified; show unavailable operations honestly until then.

Keep networking, full PACS/worklist workflows, extensive reporting, segmentation and complete workstation equivalence outside this first demonstrator slice. The initial UI is a consumer and feedback mechanism for the libraries. Every stage must remain executable through a headless library or CLI consumer.

Acceptance includes independent known-value cases, malformed-input and resource-limit behaviour, signed/meaningful-bit preservation, repeated load/release, stale-result suppression, cancellation during native work and clean-environment packaging. Execute the desktop packages on all four named desktop targets and the declared Android capabilities through Kotlin/JNI on the selected device profile, including section 14.3's page-size and packaging checks. Record compile-only or emulated results separately. A full-portfolio or clinical-conformance claim is not an acceptance shortcut.

## 17. Copy-ready coding-agent instruction

```text
You are implementing the module and milestone identified in the accompanying
work order for DICOMflux, the Flux Imaging Suite or DICOM Capacitor.

Read DICOMflux and Flux Imaging Suite Development Manifesto v1.3.1, the
repository instructions, the work order, and the actual pinned source and
contract references before changing public interfaces.

Prioritise the Rust libraries and their useful headless capabilities.
Implement the assigned DICOM, codec or resampling behaviour independently in
Rust under the project's MIT licensing direction. Use standards for required
behaviour and DICOMkit/SIS for approved API and behavioural continuity.
Record consulted or reused sources and applicable external contribution
assignments. Verify actual ownership and licences before adapting Mayam files. Do not copy or translate restricted
implementation code, hide an external DICOM/codec engine behind our API,
or use an oracle as a production fallback.

Produce the affected Swift-to-Rust API equivalence entries and preserve
semantics through idiomatic types, names, ownership and errors. Keep generic
imaging independent of DICOM datasets. For Swift bindings, verify the
covered facade and both compile-time core selections, including link inputs.

Keep parsing, codecs and applicable processing useful through the portable
CPU baseline. Qualified platform primitives may accelerate the specified
operations under the manifesto's rules. DICOMflux-3D may use admitted
rendering dependencies and require a qualified GPU without imposing graphics
requirements on the core. Apple 3D integration uses Voxelia.

Implement one useful bounded slice with explicit memory/concurrency budgets,
checked arithmetic, typed failures, predictable cancellation and output
validity. The initial DICOM Capacitor uses Java/Swing and public library
interfaces, with native operations off the EDT and explicit lifetime handling.
Qualify Windows/Linux x64 and ARM64 desktop packages. Android is also a
primary library target: provide the assigned Kotlin/JNI facade and NDK-built
Rust libraries, with AAR/consumer packaging, explicit native buffer lifetimes,
lifecycle/cancellation handling, physical-device execution and the specified
4 KB/16 KB page-size checks. An Android DICOM Capacitor application is not
required. Android DICOM Compressor is not excluded by this decision. Keep DICOM Capacitor's initial product on desktop; use a small
Android validation consumer for the library contract.

Keep macOS compatibility, C#/Swift additions and embedded 3D on their assigned
later milestones. Use the work order's selected desktop JDK/binding route;
the manifesto recommends Java 25 LTS and FFM for desktop. Android uses its
own Kotlin/JNI/NDK profile and does not inherit desktop FFM or Swing.
Implement the optional synthetic demonstration only when assigned, with
synthetic data.

Verify against justified independent expectations, appropriate comparison
implementations, malformed inputs, lifetime/concurrency checks and required
target executions. Use versioned rights-cleared fixtures. Measure complete
costs and identify the selected core and accelerator.

Continue the work authorised by the work order. Isolate exact blockers and
continue unaffected work. Do not invent missing contracts, expand scope,
weaken tests or claim unexecuted evidence.

Return reviewable changes, current capability/API/provenance records, exact
commands and observed results, source/build/fixture identities, measurements,
limitations, SHA-256 integrity records and the next bounded step. Distinguish
compilation from execution and module verification from product qualification.
```

## 18. Sources and reference handling

The links below support the external facts and reference rules in this manifesto. Sources S01-S25 and I01 were consulted on 8 November 2026; S01 and the Java, licensing and demonstration additions S26-S40 were checked on 9 November 2026. The Android additions S41-S45 were checked on 9 November 2026. Moving documentation URLs are discovery points; module agents must retain the actual versions and digests they use. Programme requirements above are engineering policy, not quotations from these sources.

| ID | Source and use |
| --- | --- |
| S01 | [Official DICOM current publication index](https://dicom.nema.org/medical/DICOM/current/) - consolidated 2026d publication observed at issue |
| S02 | [DICOM PS3.1 section 1.4.2, Continuous Maintenance](https://dicom.nema.org/medical/dicom/current/output/chtml/part01/sect_1.4.2.html) - Final Text changes and consolidation |
| S03 | [DICOM PS3.1 chapter 7, Referencing the DICOM Standard](https://dicom.nema.org/medical/dicom/current/output/chtml/part01/chapter_7.html) and [PS3.2 chapter 6](https://dicom.nema.org/medical/dicom/current/output/chtml/part02/chapter_6.html) - feature/role claims and conformance statements |
| S04 | [MIT licence, Open Source Initiative](https://opensource.org/license/mit) - reuse permission and notice condition |
| S05 | [AGPLv3, Open Source Initiative](https://opensource.org/license/agpl-3-0) - definitions and obligations for covered adaptations; not a blanket conclusion about mere reference use |
| S06 | [Rust API Guidelines, Naming](https://rust-lang.github.io/api-guidelines/naming.html) - idiomatic naming and ownership-sensitive conversion conventions |
| S07 | [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html) - target support categories, distinct from our qualification |
| S08 | [Cargo rust-version](https://doc.rust-lang.org/cargo/reference/rust-version.html) and [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html) - supported compiler contract and feature/dependency management |
| S09 | [DICOM PS3.5 section 7.8, Private Data Elements](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/sect_7.8.html) - private-creator handling and item scope |
| S10 | [DICOM PS3.5 section 8.2.15](https://dicom.nema.org/Medical/Dicom/current/output/chtml/part05/sect_8.2.15.html) and [section 10.19](https://dicom.nema.org/MEDICAL/DICOM/current/output/chtml/part05/sect_10.19.html) - JPEG XL representation and reconstruction distinctions |
| S11 | [JPEG committee HTJ2K overview](https://jpeg.org/jpeg2000/htj2k.html) - mathematically lossless recoding capability; implementation profiles remain separately qualified |
| S12 | [Rust external ABI reference](https://doc.rust-lang.org/reference/items/external-blocks.html) and [Rust linkage](https://doc.rust-lang.org/reference/linkage.html) - foreign interfaces and system-library outputs |
| S13 | [Rust catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html) - limits of panic containment |
| S14 | [Rust MaybeUninit](https://doc.rust-lang.org/std/mem/union.MaybeUninit.html) and [slice construction safety](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) - initialisation and validity of safe views over storage |
| S15 | [SwiftPM package configuration](https://docs.swift.org/package-manager/PackageDescription/PackageDescription.html), [C module maps](https://docs.swift.org/main/documentation/packagemanagerdocs/modulemaps/) and [Apple binary-framework distribution](https://developer.apple.com/documentation/Xcode/distributing-binary-frameworks-as-swift-packages) - Swift wrapper, package and binary-variant mechanisms |
| S16 | [Apple Accelerate](https://developer.apple.com/documentation/accelerate) and [Metal resource synchronisation](https://developer.apple.com/documentation/metal/resource-synchronization) - supported numerical facilities and explicit resource access/completion |
| S17 | [NVIDIA CUDA Unified Memory](https://docs.nvidia.com/cuda/cuda-programming-guide/04-special-topics/unified-memory.html) - platform-dependent memory behaviour and migration |
| S18 | [Qualcomm Adreno Vulkan/OpenCL sharing](https://www.qualcomm.com/developer/blog/2026/10/how-to-share-vulkan-opencl-workloads-android-adreno-gpus) and [Hexagon kernel tooling](https://www.qualcomm.com/developer/blog/2026/02/build-faster-on-hexagon-npu-tritor-pytorch-with-hexagon-mlir-open-source) - candidate interoperation and compute paths requiring specific qualification |
| S19 | [Intel oneAPI compiler release notes](https://www.intel.com/content/www/us/en/developer/articles/release-notes/oneapi-dpcpp/2026.html) and [OpenVINO NPU requirements](https://docs.openvino.ai/2026/openvino-workflow/running-inference/inference-devices-and-modes/npu-device.html) - separate GPU/compiler and NPU support conditions |
| S20 | [AMD ROCm support and release notes](https://rocm.docs.amd.com/en/latest/about/release-notes.html) and [Ryzen AI deployment](https://ryzenai.docs.amd.com/en/latest/modelrun.html) - configuration-specific GPU and NPU deployment paths |
| S21 | [wgpu project and platform table](https://github.com/gfx-rs/wgpu), [licence declaration](https://github.com/gfx-rs/wgpu/blob/trunk/Cargo.toml) and [adapter capabilities](https://docs.rs/wgpu/latest/wgpu/struct.Adapter.html) - graphics backends, permissive licensing and feature/limit queries |
| S22 | [egui licence declaration](https://github.com/emilk/egui/blob/main/Cargo.toml) and [egui-wgpu integration](https://docs.rs/egui-wgpu/latest/egui_wgpu/) - UI licensing and custom renderer integration |
| S23 | [DICOM PS3.3 C.7.6.2, Image Plane Module](https://dicom.nema.org/medical/dicom/current/output/chtml/part03/sect_C.7.6.2.html) - patient coordinates, pixel spacing, orientation, position and slice-spacing distinctions |
| S24 | [DICOM Secondary Capture IOD](https://dicom.nema.org/medical/dicom/current/output/chtml/part03/sect_A.8.html), [Synthetic Data attribute](https://dicom.nema.org/medical/dicom/current/output/chtml/part03/sect_C.12.html), [SC Conversion Type](https://dicom.nema.org/medical/dicom/current/output/chtml/part03/sect_C.8.6.html) and [Part 10 file format](https://dicom.nema.org/medical/dicom/current/output/chtml/part10/chapter_7.html) - valid synthetic demonstration objects |
| S25 | [DICOM UID rules](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/chapter_9.html), [UID registry](https://dicom.nema.org/medical/dicom/current/output/chtml/part06/chapter_A.html) and [JPEG XL encapsulation](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/sect_A.4.12.html) - instance identity and coding-process/Transfer-Syntax distinctions |
| S26 | [Avalonia FAQ](https://avaloniaui.net/faq) and [core MIT licence](https://github.com/AvaloniaUI/Avalonia/blob/main/licence.md) - free core and separately licensed commercial products |
| S27 | [OSI FAQ](https://opensource.org/faq) - commercial use, continuing fork rights, copyright assignment versus contributor licensing and preservation of ownership notices |
| S28 | [Apache contributor agreements](https://apache.org/licenses/contributor-agreements.html) and [DCO 1.1](https://developercertificate.org/) - licensing and provenance certifications distinguished from assignment |
| S29 | [Microsoft OpenJDK downloads](https://learn.microsoft.com/en-us/java/openjdk/download), [support policy](https://learn.microsoft.com/en-us/java/openjdk/support) and [licensing FAQ](https://learn.microsoft.com/en-us/java/openjdk/faq) - Java 25 LTS native targets, servicing and GPLv2 with Classpath Exception |
| S30 | [Java 25 FFM guide](https://docs.oracle.com/en/java/javase/25/core/foreign-function-and-memory-api.html), [restricted methods](https://docs.oracle.com/en/java/javase/25/core/restricted-methods.html) and [Java 22 release notes](https://www.oracle.com/java/technologies/javase/22-relnote-issues.html) - standard FFM, native invocation and launcher permissions |
| S31 | [Java 25 java.desktop module](https://docs.oracle.com/en/java/javase/25/docs/api/java.desktop/module-summary.html) - Swing/AWT desktop components |
| S32 | [Java 25 Arena](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/lang/foreign/Arena.html) - memory lifetime and Java-thread access constraints |
| S33 | [Java 25 SwingWorker](https://docs.oracle.com/en/java/javase/25/docs/api/java.desktop/javax/swing/SwingWorker.html) - background work, EDT updates and unspecified default scheduling |
| S34 | [Java 25 JAWT specification](https://docs.oracle.com/en/java/javase/25/docs/specs/AWT_Native_Interface.html) - supported native graphics integration, JNI dependency and platform surface contracts |
| S35 | [wgpu native surface targets](https://docs.rs/wgpu/latest/wgpu/enum.SurfaceTargetUnsafe.html) - compatible native handles and their required lifetime |
| S36 | [Java 25 BufferedImage](https://docs.oracle.com/en/java/javase/25/docs/api/java.desktop/java/awt/image/BufferedImage.html) - image storage formats and getRGB conversion precision |
| S37 | [DICOM PS3.5 section 6.2](https://dicom.nema.org/medical/dicom/current/output/chtml/part05/sect_6.2.html) - PN components, text VR lengths and character repertoire |
| S38 | [Java 25 packaging overview](https://docs.oracle.com/en/java/javase/25/jpackage/packaging-overview.html) - jpackage, bundled runtime, platform packaging and runtime module closure |
| S39 | [WiX maintenance-fee policy](https://docs.firegiant.com/wix/osmf/) and [release notes](https://docs.firegiant.com/wix/whatsnew/releasenotes/) - current distribution terms and explicit EULA requirements, separate from the source licence |
| S40 | [Oracle-hosted Classpath Exception text](https://docs.oracle.com/cd/E65412_01/E71326/html/license-classpath-exception.html) - independent-module exception; explanatory licence text, not the terms of every Oracle-branded JDK download |
| S41 | [Android JNI tips](https://developer.android.com/ndk/guides/jni-tips) and [Java versions in Android builds](https://developer.android.com/build/jdks) - managed/native boundaries, JNI thread/reference rules and the build-JDK versus Android-runtime distinction |
| S42 | [Android ABIs](https://developer.android.com/ndk/guides/abis) and [Rust Android target support](https://doc.rust-lang.org/rustc/platform-support/android.html) - Android ABI names, Rust targets and NDK-based cross-compilation |
| S43 | [Create an Android library](https://developer.android.com/studio/projects/android-library) - AAR contents, ABI-specific native libraries, consumer dependencies and shrinker rules |
| S44 | [Android support for 16 KB page sizes](https://developer.android.com/guide/practices/page-sizes) - ELF and RELRO layout, final APK native-library alignment, runtime assumptions and execution checks |
| S45 | [Android ContentResolver](https://developer.android.com/reference/android/content/ContentResolver) - content-URI access, descriptor ownership and possible non-seekable input |
| I01 | [SwiftRLE coding-agent instructions](https://chatgpt.com/library?file_id=libfile_f3d3fc23e1a88191912bb7c51b3d78ba&account_id=personal), version 1.0.0, 22 September 2026 - consulted sections on common API, memory lifecycle, independence and cross-codec behaviour; historical terminology and pins must be reconciled with the actual baseline assigned to each agent |

This issue does not freeze an uninspected current DICOMkit/SIS or Voxelia symbol catalogue. The API equivalence and interoperability procedures are mandatory so that each implementation follows verified source declarations and accepted contracts. External DICOM/codec engine names identify reference categories, not admitted shipping engines. Graphics/UI dependencies and platform primitives are admitted only within the explicit rules above.

## 19. Version record

**1.0.0 - 8 November 2026:** initial manifesto based on the owner's independent Rust/MIT direction, adding explicit standards maintenance, provenance, API equivalence, memory/concurrency, fidelity, platform, verification, packaging and coding-agent completion rules.

**1.1.0 - 8 November 2026:** fresh Rust programme identity; compile-time Swift core substitution and comparison; explicit hardware/OS backends; discrete DICOMflux-3D and Apple Voxelia integration; DICOM Capacitor delivery sequence; restrained documentation references and an optional synthetic DICOM demonstration.

**1.2.0 - 9 November 2026:** Rust libraries first; initial Java/Swing DICOM Capacitor; Windows/Linux x64 and ARM64 first release matrix; secondary Mac/Swift compatibility; recommended Java 25/FFM baseline; explicit Swing threading, presentation and packaging contracts; separate 3D integration milestone; external copyright assignment and commercial-distribution policy; optional fictional-character and quotation presets.

**1.3.0 - 9 November 2026:** Android added as a primary DICOMflux and Flux Imaging Suite platform with Kotlin/JNI and Android NDK integration. Added recommended ARM64 device and x86_64 emulator profiles, AAR/consumer packaging, lifecycle and buffer-ownership rules, storage adapters, 4 KB/16 KB execution requirements and first-release Android gates. Desktop Java/Swing DICOM Capacitor retains its Windows/Linux scope. An Android DICOM Capacitor application is not required; Android DICOM Compressor remains a separate application decision; Android codec and processing library capabilities remain in scope.

**1.3.1 - 9 November 2026 (scope correction):** Corrected the Android product exclusion throughout: DICOM Capacitor, not DICOM Compressor, is not required on Android. The Android DICOM Compressor application remains a separate decision. DICOM Camera and Oviyam | Mobile are intended Android consumers. Android 3D rendering and the choice between Kotlin-native or Rust-native UI are architecture investigations, not approved implementation changes.

Maintain this as one common versioned manifesto. Module work orders specialise its rules and record their pinned version. A correction affecting a shared interface or guarantee must be communicated to every affected module; individual agents must not create incompatible local variants of the common policy.

