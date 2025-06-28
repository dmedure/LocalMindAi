use std::env;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to rerun this build script if these files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    // Get the target platform
    let target = env::var("TARGET").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    println!("cargo:rustc-env=TARGET_PLATFORM={}", target);
    println!("cargo:rustc-env=TARGET_OS={}", target_os);
    println!("cargo:rustc-env=TARGET_ARCH={}", target_arch);
    
    // ============================================================================
    // DIOXUS DESKTOP CONFIGURATION
    // ============================================================================
    
    // Set up Dioxus desktop resources
    if cfg!(feature = "desktop") || env::var("CARGO_FEATURE_DESKTOP").is_ok() {
        setup_dioxus_desktop();
    }
    
    // ============================================================================
    // AI LIBRARY CONFIGURATION
    // ============================================================================
    
    // Configure AI libraries based on features
    setup_ai_libraries();
    
    // ============================================================================
    // PLATFORM-SPECIFIC CONFIGURATIONS
    // ============================================================================
    
    match target_os.as_str() {
        "windows" => setup_windows(),
        "macos" => setup_macos(),
        "linux" => setup_linux(),
        _ => println!("cargo:warning=Unsupported target OS: {}", target_os),
    }
    
    // ============================================================================
    // OPTIMIZATION FLAGS
    // ============================================================================
    
    setup_optimization_flags();
    
    // ============================================================================
    // RESOURCE MANAGEMENT
    // ============================================================================
    
    setup_resources();
}

fn setup_dioxus_desktop() {
    println!("cargo:rustc-cfg=desktop");
    
    // Set up desktop app metadata
    println!("cargo:rustc-env=APP_NAME=Jinnie AI");
    println!("cargo:rustc-env=APP_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=APP_DESCRIPTION={}", env!("CARGO_PKG_DESCRIPTION"));
    
    // Windows-specific desktop setup
    if cfg!(target_os = "windows") {
        // Set up Windows app icon and manifest
        println!("cargo:rustc-link-search=native=./resources/windows");
        
        // Enable Windows subsystem for GUI app (no console window)
        if env::var("PROFILE").unwrap() == "release" {
            println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        }
    }
    
    // macOS-specific desktop setup
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=framework=./resources/macos");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    }
    
    // Linux-specific desktop setup
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-search=native=./resources/linux");
        
        // GTK and WebKit dependencies for Linux
        pkg_config_check("gtk+-3.0");
        pkg_config_check("webkit2gtk-4.0");
    }
}

fn setup_ai_libraries() {
    // Candle configuration
    if env::var("CARGO_FEATURE_CANDLE_CORE").is_ok() {
        println!("cargo:rustc-cfg=feature=\"candle\"");
        
        // Enable CUDA support if available
        if env::var("CUDA_PATH").is_ok() || env::var("CUDA_HOME").is_ok() {
            println!("cargo:rustc-cfg=cuda");
            println!("cargo:rustc-env=CANDLE_USE_CUDA=1");
        }
        
        // Enable Metal support on macOS
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-cfg=metal");
            println!("cargo:rustc-env=CANDLE_USE_METAL=1");
        }
        
        // Set up OpenBLAS or similar for CPU acceleration
        setup_blas_backend();
    }
    
    // PyTorch (tch) configuration
    if env::var("CARGO_FEATURE_TCH").is_ok() {
        setup_pytorch();
    }
    
    // ONNX Runtime configuration
    if env::var("CARGO_FEATURE_ORT").is_ok() {
        setup_onnx_runtime();
    }
    
    // Vector database configuration
    if env::var("CARGO_FEATURE_QDRANT_CLIENT").is_ok() {
        println!("cargo:rustc-cfg=feature=\"vector_db\"");
    }
}

fn setup_blas_backend() {
    // Try to find and configure BLAS libraries for math acceleration
    let blas_backends = ["openblas", "netlib", "intel-mkl"];
    
    for backend in &blas_backends {
        if pkg_config::probe(backend).is_ok() {
            println!("cargo:rustc-cfg=blas=\"{}\"", backend);
            println!("cargo:rustc-env=BLAS_BACKEND={}", backend);
            return;
        }
    }
    
    // Fallback to basic implementation
    println!("cargo:rustc-cfg=blas=\"basic\"");
    println!("cargo:rustc-env=BLAS_BACKEND=basic");
}

fn setup_pytorch() {
    println!("cargo:rustc-cfg=pytorch");
    
    // Check for PyTorch installation
    if let Ok(torch_path) = env::var("TORCH_HOME") {
        println!("cargo:rustc-env=TORCH_PATH={}", torch_path);
    } else if let Ok(libtorch_path) = env::var("LIBTORCH") {
        println!("cargo:rustc-env=TORCH_PATH={}", libtorch_path);
    } else {
        println!("cargo:warning=PyTorch not found. Please set TORCH_HOME or LIBTORCH environment variable.");
    }
}

fn setup_onnx_runtime() {
    println!("cargo:rustc-cfg=onnx");
    
    // Configure ONNX Runtime providers
    let mut providers = vec!["cpu"];
    
    if env::var("CUDA_PATH").is_ok() {
        providers.push("cuda");
    }
    
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        providers.push("coreml");
    }
    
    println!("cargo:rustc-env=ONNX_PROVIDERS={}", providers.join(","));
}

fn setup_windows() {
    println!("cargo:rustc-cfg=windows_platform");
    
    // Windows-specific library paths
    if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
        let lib_path = format!("{}/installed/x64-windows/lib", vcpkg_root);
        println!("cargo:rustc-link-search=native={}", lib_path);
    }
    
    // Windows multimedia libraries for audio
    if env::var("CARGO_FEATURE_AUDIO").is_ok() {
        println!("cargo:rustc-link-lib=winmm");
        println!("cargo:rustc-link-lib=dsound");
        println!("cargo:rustc-link-lib=ole32");
    }
}

fn setup_macos() {
    println!("cargo:rustc-cfg=macos_platform");
    
    // macOS system frameworks
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");
    
    // Audio frameworks for macOS
    if env::var("CARGO_FEATURE_AUDIO").is_ok() {
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
    }
    
    // Metal for GPU acceleration
    if env::var("CARGO_FEATURE_CANDLE_CORE").is_ok() {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    }
}

fn setup_linux() {
    println!("cargo:rustc-cfg=linux_platform");
    
    // Audio libraries for Linux
    if env::var("CARGO_FEATURE_AUDIO").is_ok() {
        pkg_config_check("alsa");
        pkg_config_check("pulse");
        pkg_config_check("jack");
    }
    
    // OpenGL libraries
    pkg_config_check("gl");
    pkg_config_check("egl");
}

fn setup_optimization_flags() {
    let profile = env::var("PROFILE").unwrap();
    
    match profile.as_str() {
        "release" => {
            // Release optimizations
            println!("cargo:rustc-cfg=optimized");
            println!("cargo:rustc-env=BUILD_MODE=release");
            
            // Enable CPU-specific optimizations
            if cfg!(target_arch = "x86_64") {
                println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=native");
            }
        }
        "dev" => {
            // Development optimizations
            println!("cargo:rustc-cfg=debug_build");
            println!("cargo:rustc-env=BUILD_MODE=debug");
            
            // Enable faster compilation
            println!("cargo:rustc-env=RUSTFLAGS=-C incremental=y");
        }
        _ => {}
    }
}

fn setup_resources() {
    // Set up resource directories
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    println!("cargo:rustc-env=RESOURCE_DIR={}/resources", manifest_dir);
    println!("cargo:rustc-env=ASSETS_DIR={}/assets", manifest_dir);
    println!("cargo:rustc-env=OUT_DIR={}", out_dir);
    
    // Create resource directories if they don't exist
    let resource_dirs = ["resources", "assets", "models", "data"];
    for dir in &resource_dirs {
        let path = PathBuf::from(&manifest_dir).join(dir);
        if !path.exists() {
            let _ = std::fs::create_dir_all(&path);
        }
    }
}

fn pkg_config_check(lib: &str) {
    match pkg_config::probe(lib) {
        Ok(_) => {
            println!("cargo:rustc-cfg=has_{}", lib.replace("-", "_"));
        }
        Err(_) => {
            println!("cargo:warning=Library {} not found via pkg-config", lib);
        }
    }
}