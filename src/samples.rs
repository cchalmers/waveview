const SAMPLES: &[&str] = &[
// GitHub

// `github=<user>/<repo>/<brunch>/<filename>.vcd`

// - Icarus
"github=dpretet/vcd/master/test1.vcd",
"github=ombhilare999/riscv-core/master/src/rv32_soc_TB.vcd",
"github=b06902044/computer_architecture/main/CPU.vcd",

// - Verilator
"github=wavedrom/vcd-samples/trunk/swerv1.vcd", // big vcd, lots of bugs here
"github=bigBrain1901/nPOWER-ISA-5-STAGE-PIPELINED-CPU/master/post_compile_files/vlt_dump.vcd",

// - GHDL
"github=AdoobII/idea_21s/main/vhdl/idea.vcd",
"github=yne/vcd/master/plasma.vcd",
"github=yne/vcd/master/sample.vcd",
"github=charlycop/VLSI-1/master/EXEC/ALU/alu.vcd",
"github=gaoqqt2n/CPU/master/SuperPipelineCPU/vcdfile/pcpu.vcd",


// - VCS
"github=sathyapriyanka/APB_UVC_UVM/5401170f6c74453c83f24df06ce9228c198f6d20/Apb_slave_uvm_new.vcd",
"github=Akashay-Singla/RISC-V/main/Pipeline/datapath_log.vcd",
"github=Akashay-Singla/RISC-V/main/2_way_Superscalar/datapath_log.vcd",
"github=ameyjain/8-bit-Microprocessor/master/8-bit%20microprocessor/processor.vcd",

// - QuestaSim
"github=mr-gaurav/Sequence-Counter/main/test.vcd",
"github=SparshAgarwal/Computer-Architecture/master/hw3/hw3_1/dump.vcd",

// - ModelSim
"github=sh619/Songyu_Huang-Chisel/main/MU0_final_version/simulation/qsim/CPU_Design.msim.vcd",

// - QUARTUS_VCD_EXPORT
"github=PedroTLemos/ProjetoInfraHard/master/mipsHardware.vcd",

// - SystemC
"github=jroslindo/Mips-Systemc/main/REGISTRADORES_32_bits/wave_registradores.vcd",
"github=amrhas/PDRNoC/VCRouter/noctweak/Debug/wavform.vcd.vcd",

// - Xcelium (xmsim)
"github=avidan-efody/wave_rerunner/main/test/data/integrated.vcd", // invalid scope name

// - treadle
"github=chipsalliance/treadle/master/src/test/resources/GCD.vcd",

// - Riviera-PRO
"github=prathampathak/Tic-Tac-Tao/main/dump.vcd",

// - MyHDL
"github=aibtw/myHdl_Projects/main/SimpleMemory/Simple_Memory.vcd",
"github=Abhishek010397/Programming-RISC-V/master/top.vcd",
"github=DarthSkipper/myHDL_Sigmoid/master/out/testbench/sigmoid_tb.vcd",

// - ncsim
"github=amiteee78/RTL_design/master/ffdiv_32bit/ffdiv_32bit_prop_binom/run_cad/ffdiv_32bit_tb.vcd", // slow

// - xilinx_isim
"github=mukul54/qrs-peak-fpga/master/utkarsh/utkarsh.sim/sim_1/behav/xsim/test.vcd",
"github=DanieleParravicini/regex_coprocessor/master/scripts/sim/test2x2_regex22_string1.vcd",
"github=pabloec1729/Hashes-generator/master/RTL/velocidad/test.vcd",

// Vivado
"github=saharmalmir/Eth2Ser/master/UART2ETH.runs/impl_1/iladata.vcd",
"github=BradMcDanel/multiplication-free-dnn/master/verilog/iladata.vcd",

// GTKWave Analyzer
"github=Asfagus/Network-Switch/main/perm_current.vcd", // (Late start. t0 > 0)

// Gist

// `gist=<user>/<hash>/raw/<hash>/<filename>.vcd`

// https://gist.github.com/dougallj/7a75a3be1ec69ca550e7c36dc75e0d6f
// https://gist.githubusercontent.com/dougallj/7a75a3be1ec69ca550e7c36dc75e0d6f/raw/81e4648d9909c3b1f0943f88896b81101510aa70/aarch64_amx.py

"gist=drom/3b5f2ba5e2f60a91f9a8e765727858fe/raw/f79178d9e573d0957c065880b942882710a1660d/test1.vcd", // (Icarus)

"gist=carlosedp/00380f29bbd7aadc3523ffd162230d0e/raw/d732be78edb558ba91df5b3b1475288279df96fd/Blinky.vcd", // (Treadle)

// Bitbucket

// `bitbucket=<user>/<repo>/raw/<hash>/<filename>.vcd`

"bitbucket=alex_drom/vcd-samples/raw/36cf049c82f70f82249682d20444903627b9536e/test1.vcd", // (Icarus)

// GitLab

// :construction: Does not work Yet :construction:

// `Cross-Origin Request Blocked`

// `gitlab=<user>/<repo>/<brunch>/<filename>.vcd`

"gitlab=drom/vcd-samples/raw/main/swerv1.vcd", // (Verilator)
    // long (causes render errors)

// Snippets

"https://gitlab.com/-/snippets/2162111/raw/main/test1.vcd",
];

fn parse_sample(sample: &str) -> Option<String> {
    let mut parts = sample.split('=');
    let host = parts.next().unwrap();
    let url = if let Some(path) = parts.next() {
        assert!(parts.next().is_none());
        match host {
            "github" => {
                // "github=dpretet/vcd/master/test1.vcd",
                let mut parts = path.split('/');
                let user = parts.next().unwrap();
                let repo = parts.next().unwrap();
                let branch = parts.next().unwrap_or("master");
                let file = parts.collect::<Vec<_>>().join("/");
                format!(
                    "https://raw.githubusercontent.com/{}/{}/{}/{}",
                    user, repo, branch, file
                )
            }
            "gist" => {
                // let mut parts = path.split('/');
                // let user = parts.next().unwrap();
                // let hash = parts.next().unwrap();
                // let file = parts.collect::<Vec<_>>().join("/");
                // format!(
                //     "https://gist.githubusercontent.com/{}/raw/{}/{}",
                //     user, hash, file
                // )
                format!("https://gist.githubusercontent.com/{}", path)
            }
            "bitbucket" => {
                // let mut parts = path.split('/');
                // let user = parts.next().unwrap();
                // let repo = parts.next().unwrap();
                // let hash = parts.next().unwrap();
                // let file = parts.collect::<Vec<_>>().join("/");
                // format!(
                //     "https://bitbucket.org/{}/{}/raw/{}/{}",
                //     user, repo, hash, file
                // )
                format!("https://bitbucket.org/{}", path)
            }
            "gitlab" => {
                // let mut parts = path.split('/');
                // let user = parts.next().unwrap();
                // let repo = parts.next().unwrap();
                // let branch = parts.next().unwrap_or("master");
                // let file = parts.collect::<Vec<_>>().join("/");
                // format!(
                //     "https://gitlab.com/{}/{}/{}/{}",
                //     user, repo, branch, file
                // )
                format!("https://gitlab.com/{}", path)
            }
            _ => return None,
        }
    } else {
        host.to_owned()
    };
    Some(url)
}

use eframe::egui::*;

pub fn show_samples(ui: &mut Ui) -> Option<String> {
    // let mut samples = SAMPLES.to_vec();
    // samples.sort();
    let mut chosen_sample = None;
    for sample in SAMPLES {
        let name = sample.split('/').last().unwrap();
        if ui.button(name).clicked() {
            chosen_sample = Some(sample);
        }
    }
    if let Some(sample) = chosen_sample {
        parse_sample(sample)
    } else {
        None
    }
}
