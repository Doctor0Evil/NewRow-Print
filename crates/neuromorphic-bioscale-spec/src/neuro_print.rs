neuro.print!(
    neuromorphic_bioscale_spec.v2026_02 {
        thermal.envelope {
            core_c_max    = 37.8;
            iface_delta_c = 0.7;
            abort_delta_c = 2.0;
        }
        energy.synapse {
            class.bio_proximal { esyn_fj_min = 0.05; esyn_fj_max = 1.0; }
            class.edge_accel  { esyn_pj_min = 0.2;  esyn_pj_max = 1.0; }
            class.legacy_cmos { esyn_pj_min = 10.0; esyn_pj_max = 400.0; }
        }
        bio.interface {
            material.graphene_blast  = "soft, Nafion+graphene, neuromorphic synapse";
            material.droplet_synapse = "ionic DIS, 4–8 pJ/spike";
            material.metal_mea       = "tRTD-MEA, low-noise, cytotox-safe";
        }
        algo.envelope {
            max_power_mw_implant = 10.0;
            esyn_target_pj       = 0.2;
            spike_rate_hz_max    = 1_000.0;
        }
        evidence.hex {
            cortical_heating      = "a1f3c9b2";
            rf_heating_eeg_mri    = "2f8c6b44";
            graphene_synapse_ef   = "6ac2f9d9";
            droplet_synapse_pj    = "9cd4a7e8";
        }
    }
);
