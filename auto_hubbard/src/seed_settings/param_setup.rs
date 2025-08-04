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
use sealed::Sealed;
use serde::{Deserialize, Serialize};

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
    calculate_elf: CalculateElf,
    calculate_stress: CalculateStress,
    popn_calculate: PopnCalculate,
    calculate_hirshfeld: CalculateHirshfeld,
    calculate_densdiff: CalculateDensdiff,
    popn_bond_cutoff: PopnBondCutoff,
    pdos_calculate_weights: PdosCalculateWeights,
    iprint: Iprint,
}

pub struct HubbardUParam<T: Stage> {
    param: ParamFile,
    stage: PhantomData<T>,
}

pub trait Stage: Sealed {}

pub struct Init;
pub struct BeforePerturb;
pub struct Perturbed;

impl HubbardUParam<Init> {
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
    /// The `elec_energy_tol` will be divided by 10
    pub fn param_after_perturb(&self) -> HubbardUParam<Perturbed> {
        HubbardUParam {
            param: ParamFile {
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

mod sealed {
    use super::{BeforePerturb, Init, Perturbed, Stage};

    pub(super) trait Sealed {}
    impl Sealed for Init {}
    impl Sealed for BeforePerturb {}
    impl Sealed for Perturbed {}
    impl Stage for Init {}
    impl Stage for BeforePerturb {}
    impl Stage for Perturbed {}
}

#[cfg(test)]
mod test {}
