use std::marker::PhantomData;

use castep_cell_data::{
    param::{
        basis_set::{CutOffEnergy, FineGridScale, FiniteBasisCorr, GridScale},
        density_mixing::{
            MixChargeAmp, MixChargeGmax, MixHistoryLength, MixSpinAmp, MixSpinGmax, MixingScheme,
        },
        electronic::{NextraBands, Spin},
        electronic_minimisation::{
            ElecEnergyTol, FixOccupancy, MaxScfCycles, MetalsMethod, NumDumpCycles, SmearingWidth,
            SpinFix,
        },
        exchange_correlation::{SpinPolarized, XcFunctional},
        general::{
            CalculateDensdiff, CalculateElf, CalculateHirshfeld, CalculateStress, Continuation,
            Iprint, OptStrategy, Task,
        },
        population_analysis::{PdosCalculateWeights, PopnBondCutoff, PopnCalculate},
    },
    ToCellFileDerive,
};
use serde::{Deserialize, Serialize};

use super::{BeforePerturb, Init, Perturbed, Stage};

#[derive(Debug, Clone, Serialize, Deserialize, ToCellFileDerive)]
pub struct ParamFile {
    task: Task,
    continuation: Option<Continuation>,
    xc_functional: XcFunctional,
    spin_polarized: SpinPolarized,
    spin: Spin,
    opt_strategy: OptStrategy,
    cut_off_energy: CutOffEnergy,
    grid_scale: GridScale,
    fine_grid_scale: Option<FineGridScale>,
    finite_basis_corr: FiniteBasisCorr,
    elec_energy_tol: ElecEnergyTol,
    max_scf_cycles: MaxScfCycles,
    fix_occupancy: FixOccupancy,
    metals_method: MetalsMethod,
    mixing_scheme: MixingScheme,
    mix_charge_amp: MixChargeAmp,
    mix_spin_amp: MixSpinAmp,
    mix_charge_gmax: MixChargeGmax,
    mix_spin_gmax: MixSpinGmax,
    mix_history_length: MixHistoryLength,
    nextra_bands: NextraBands,
    smearing_width: SmearingWidth,
    spin_fix: SpinFix,
    num_dump_cycles: NumDumpCycles,
    #[serde(alias = "calculate_ELF")]
    calculate_elf: CalculateElf,
    calculate_stress: CalculateStress,
    popn_calculate: PopnCalculate,
    calculate_hirshfeld: CalculateHirshfeld,
    calculate_densdiff: CalculateDensdiff,
    popn_bond_cutoff: PopnBondCutoff,
    pdos_calculate_weights: PdosCalculateWeights,
    iprint: Iprint,
}

#[derive(Debug, Clone)]
pub struct HubbardUParam<T: Stage> {
    pub param: ParamFile,
    stage: PhantomData<T>,
}

impl HubbardUParam<Init> {
    pub fn from_param(param: ParamFile) -> Self {
        Self {
            param,
            stage: PhantomData,
        }
    }
    /// Create a new `.param` for our task.
    pub fn param_before_perturb(
        &self,
        init_elec_energy_tol: ElecEnergyTol,
    ) -> HubbardUParam<BeforePerturb> {
        let new_param = ParamFile {
            continuation: None,
            elec_energy_tol: init_elec_energy_tol,
            fine_grid_scale: None,
            grid_scale: GridScale(1.75),
            ..self.param.clone()
        };
        HubbardUParam {
            param: new_param,
            stage: PhantomData,
        }
    }
}

impl HubbardUParam<BeforePerturb> {
    /// Create a new `.param` for perturbation.
    /// `continuation` is set to `default`
    /// The `elec_energy_tol` will be divided by 10
    pub fn param_after_perturb(&self) -> HubbardUParam<Perturbed> {
        HubbardUParam {
            param: ParamFile {
                continuation: Some(Continuation("default".to_string())),
                elec_energy_tol: ElecEnergyTol {
                    value: self.param.elec_energy_tol.value / 10.0,
                    unit: self.param.elec_energy_tol.unit,
                },
                ..self.param.clone()
            },
            stage: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use castep_cell_data::{from_str, param::electronic_minimisation::ElecEnergyTol};

    use super::{HubbardUParam, ParamFile};

    #[test]
    fn test_param() {
        let param_path = "../sh/test/GDY_111_Fe_U.param";
        let param_file = from_str::<ParamFile>(&read_to_string(param_path).unwrap())
            .map(HubbardUParam::from_param)
            .unwrap();
        dbg!(&param_file);
        let before_perturb = param_file.param_before_perturb(ElecEnergyTol {
            value: 1e-6,
            unit: None,
        });
        dbg!(&before_perturb.param.elec_energy_tol);
        assert_eq!(
            before_perturb.param.grid_scale,
            castep_cell_data::param::basis_set::GridScale(1.7500),
            "We are testing grid_scale {:?} {:?}",
            before_perturb.param.grid_scale,
            castep_cell_data::param::basis_set::GridScale(1.7500),
        );
        let after_perturb = before_perturb.param_after_perturb();
        dbg!(&after_perturb.param.continuation);
        dbg!(&after_perturb.param.elec_energy_tol);
    }
}
