# Multi-Modal Evaluation - Vision & Audio

## Overview

LLM Test Bench now provides **enterprise-grade multi-modal evaluation** capabilities, supporting:

- **Vision**: Image understanding, captioning, VQA, OCR, object detection
- **Audio**: Speech-to-text transcription, text-to-speech generation, audio classification
- **Video**: Video understanding, frame analysis (foundation for future expansion)

## Table of Contents

1. [Architecture](#architecture)
2. [Supported Modalities](#supported-modalities)
3. [Provider Support](#provider-support)
4. [Usage Examples](#usage-examples)
5. [Evaluation Metrics](#evaluation-metrics)
6. [Dataset Management](#dataset-management)
7. [Best Practices](#best-practices)
8. [API Reference](#api-reference)

---

## Architecture

### Core Components

```
┌─────────────────────────────────────────────┐
│           Multi-Modal System                │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │  Vision │  │  Audio  │  │  Video  │    │
│  │  Input  │  │  Input  │  │  Input  │    │
│  └────┬────┘  └────┬────┘  └────┬────┘    │
│       │            │            │          │
│       └────────────┴────────────┘          │
│                    │                        │
│          ┌─────────▼─────────┐             │
│          │  MultiModalRequest │             │
│          └─────────┬─────────┘             │
│                    │                        │
│          ┌─────────▼─────────┐             │
│          │     Provider      │             │
│          │   (OpenAI, etc)   │             │
│          └─────────┬─────────┘             │
│                    │                        │
│          ┌─────────▼─────────┐             │
│          │ MultiModalResponse │             │
│          └─────────┬─────────┘             │
│                    │                        │
│          ┌─────────▼─────────┐             │
│          │    Evaluators     │             │
│          │  (Vision, Audio)  │             │
│          └───────────────────┘             │
│                                             │
└─────────────────────────────────────────────┘
```

### Key Types

#### `MultiModalRequest`
Combines text, images, audio, and video into a single unified request:

```rust
pub struct MultiModalRequest {
    pub model: String,
    pub content: MultiModalContent,
    pub system: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub stream: bool,
}
```

#### `MultiModalResponse`
Contains generated content in any combination of modalities:

```rust
pub struct MultiModalResponse {
    pub id: String,
    pub model: String,
    pub content: MultiModalContent,
    pub usage: MultiModalUsage,
    pub finish_reason: FinishReason,
    pub created_at: DateTime<Utc>,
}
```

#### `MultiModalContent`
Flexible container for mixed media:

```rust
pub struct MultiModalContent {
    pub parts: Vec<ContentPart>,
}

pub enum ContentPart {
    Text(TextPart),
    Image(ImagePart),
    Audio(AudioPart),
    Video(VideoPart),
}
```

---

## Supported Modalities

### 1. Vision (Images)

#### Input Formats
- **Base64**: Encoded image data
- **URL**: Public image URLs
- **File Path**: Local image files

#### Supported Formats
- JPEG, PNG, WebP, GIF, BMP
- Up to 20MB per image (provider dependent)
- Automatic format detection

#### Vision Tasks
- **Image Captioning**: Generate descriptions
- **Visual QA**: Answer questions about images
- **OCR**: Extract text from images
- **Object Detection**: Identify and locate objects
- **Spatial Reasoning**: Understand relationships

#### Example
```rust
use llm_test_bench_core::multimodal::{ImageInput, MultiModalRequest};

// Load image from file
let image = ImageInput::from_path("cat.jpg").await?;

// Create vision request
let request = MultiModalRequest::new("gpt-4-vision-preview")
    .with_text("What's in this image?")
    .with_image(image)
    .with_max_tokens(500);

// Get response
let response = provider.complete_multimodal(request).await?;
println!("Description: {}", response.text());
```

### 2. Audio (Speech)

#### Input Formats
- **Base64**: Encoded audio data
- **URL**: Public audio URLs
- **File Path**: Local audio files

#### Supported Formats
- MP3, WAV, FLAC, Opus, AAC, OGG, M4A, WebM
- Up to 25MB per file (provider dependent)
- Sample rates: 8kHz to 48kHz

#### Audio Tasks
- **Speech-to-Text (STT)**: Transcribe speech
- **Text-to-Speech (TTS)**: Generate speech
- **Audio Classification**: Classify audio content
- **Speaker Diarization**: Separate speakers
- **Audio Translation**: Translate spoken language

#### Example
```rust
use llm_test_bench_core::multimodal::{AudioInput, TranscriptionOptions};

// Load audio file
let audio = AudioInput::from_path("speech.mp3").await?;

// Configure transcription
let options = TranscriptionOptions {
    language: Some("en".to_string()),
    timestamps: true,
    temperature: Some(0.0),
    ..Default::default()
};

// Transcribe
let transcription = provider.transcribe_audio(audio, options).await?;
println!("Transcription: {}", transcription.text);
```

### 3. Video (Frames)

#### Input Formats
- **URL**: Public video URLs
- **File Path**: Local video files
- **Frame Sequence**: Pre-extracted frames

#### Supported Formats
- MP4, WebM, AVI, MOV, MKV
- Automatic frame extraction
- Configurable sampling rate

#### Video Tasks
- **Video Understanding**: Describe video content
- **Frame Analysis**: Analyze specific frames
- **Action Recognition**: Identify actions/events
- **Scene Detection**: Segment by scene

#### Example
```rust
use llm_test_bench_core::multimodal::{VideoInput, MultiModalRequest};

// Load video
let video = VideoInput::from_path("demo.mp4").await?;

// Analyze video
let request = MultiModalRequest::new("gemini-pro-vision")
    .with_text("What happens in this video?")
    .with_video(video)
    .with_max_tokens(1000);

let response = provider.complete_multimodal(request).await?;
```

---

## Provider Support

### Provider Capabilities Matrix

| Provider | Vision | Audio (STT) | Audio (TTS) | Video | Notes |
|----------|--------|-------------|-------------|-------|-------|
| **OpenAI** | ✅ GPT-4 Vision | ✅ Whisper | ✅ TTS | ❌ | Best vision quality |
| **Anthropic** | ✅ Claude 3 | ❌ | ❌ | ❌ | Strong image reasoning |
| **Google** | ✅ Gemini | ✅ | ✅ | ✅ | Multi-modal native |
| **Azure OpenAI** | ✅ | ✅ | ✅ | ❌ | Enterprise features |
| **Together AI** | ✅ LLaVA | ❌ | ❌ | ❌ | Open source models |
| **Replicate** | ✅ | ✅ | ✅ | ✅ | Various models |

### OpenAI Vision

```rust
use llm_test_bench_core::providers::OpenAIProvider;
use llm_test_bench_core::multimodal::{ImageInput, MultiModalRequest};

let provider = OpenAIProvider::new(api_key)?;

let image = ImageInput::from_url("https://example.com/image.jpg");
let request = MultiModalRequest::new("gpt-4-vision-preview")
    .with_text("Describe this image in detail")
    .with_image(image);

let response = provider.complete_multimodal(request).await?;
```

### Anthropic Claude 3 Vision

```rust
use llm_test_bench_core::providers::AnthropicProvider;

let provider = AnthropicProvider::new(api_key);

let image = ImageInput::from_path("diagram.png").await?;
let request = MultiModalRequest::new("claude-3-opus-20240229")
    .with_text("Analyze this technical diagram")
    .with_image(image);

let response = provider.complete_multimodal(request).await?;
```

### Google Gemini Multi-Modal

```rust
use llm_test_bench_core::providers::GoogleProvider;

let provider = GoogleProvider::new(api_key)?;

// Multi-modal: text + image + audio
let image = ImageInput::from_path("scene.jpg").await?;
let audio = AudioInput::from_path("narration.mp3").await?;

let request = MultiModalRequest::new("gemini-1.5-pro")
    .with_text("Compare the image with the audio description")
    .with_image(image)
    .with_audio(audio);

let response = provider.complete_multimodal(request).await?;
```

---

## Usage Examples

### Example 1: Image Captioning

```rust
use llm_test_bench_core::multimodal::*;
use llm_test_bench_core::providers::OpenAIProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = OpenAIProvider::new(std::env::var("OPENAI_API_KEY")?)?;

    // Load image
    let image = ImageInput::from_path("cat-on-keyboard.jpg").await?;

    // Create request
    let request = MultiModalRequest::new("gpt-4-vision-preview")
        .with_text("Generate a detailed caption for this image")
        .with_image(image)
        .with_max_tokens(300);

    // Get response
    let response = provider.complete_multimodal(request).await?;

    println!("Caption: {}", response.text());
    println!("Tokens used: {}", response.usage.total_tokens);

    Ok(())
}
```

### Example 2: Visual Question Answering

```rust
async fn visual_qa() -> Result<()> {
    let provider = OpenAIProvider::new(api_key)?;

    let image = ImageInput::from_path("chart.png").await?;

    let questions = vec![
        "What type of chart is this?",
        "What is the highest value shown?",
        "What trends do you observe?",
    ];

    for question in questions {
        let request = MultiModalRequest::new("gpt-4-vision-preview")
            .with_text(question)
            .with_image(image.clone());

        let response = provider.complete_multimodal(request).await?;
        println!("Q: {}", question);
        println!("A: {}\n", response.text());
    }

    Ok(())
}
```

### Example 3: Speech Transcription

```rust
use llm_test_bench_core::multimodal::{AudioInput, TranscriptionOptions};

async fn transcribe_audio() -> Result<()> {
    let provider = OpenAIProvider::new(api_key)?;

    // Load audio file
    let audio = AudioInput::from_path("meeting-recording.mp3").await?;

    // Configure transcription
    let options = TranscriptionOptions {
        language: Some("en".to_string()),
        timestamps: true,
        word_timestamps: true,
        temperature: Some(0.0),
        response_format: Some("json".to_string()),
    };

    // Transcribe
    let result = provider.transcribe_audio(audio, options).await?;

    println!("Transcription:\n{}", result.text);

    if let Some(segments) = result.segments {
        println!("\nTimestamped segments:");
        for segment in segments {
            println!("[{:.2}s - {:.2}s] {}",
                segment.start, segment.end, segment.text);
        }
    }

    Ok(())
}
```

### Example 4: Multi-Image Analysis

```rust
async fn compare_images() -> Result<()> {
    let provider = OpenAIProvider::new(api_key)?;

    // Load multiple images
    let before = ImageInput::from_path("before.jpg").await?;
    let after = ImageInput::from_path("after.jpg").await?;

    // Create request with multiple images
    let mut content = MultiModalContent::new();
    content.add_text("Compare these two images and describe the differences:");
    content.add_image(before);
    content.add_text("versus");
    content.add_image(after);

    let request = MultiModalRequest {
        model: "gpt-4-vision-preview".to_string(),
        content,
        max_tokens: Some(500),
        ..Default::default()
    };

    let response = provider.complete_multimodal(request).await?;
    println!("Differences:\n{}", response.text());

    Ok(())
}
```

### Example 5: OCR with Validation

```rust
async fn ocr_with_validation() -> Result<()> {
    let provider = OpenAIProvider::new(api_key)?;

    let image = ImageInput::from_path("document.png").await?;

    // First: Extract text
    let ocr_request = MultiModalRequest::new("gpt-4-vision-preview")
        .with_text("Extract all text from this document. Maintain the original formatting.")
        .with_image(image.clone())
        .with_max_tokens(2000);

    let extracted_text = provider.complete_multimodal(ocr_request).await?.text();

    // Second: Validate extraction
    let validation_request = MultiModalRequest::new("gpt-4-vision-preview")
        .with_text(format!(
            "Verify this text extraction is accurate:\n\n{}",
            extracted_text
        ))
        .with_image(image)
        .with_max_tokens(100);

    let validation = provider.complete_multimodal(validation_request).await?.text();

    println!("Extracted Text:\n{}\n", extracted_text);
    println!("Validation: {}", validation);

    Ok(())
}
```

---

## Evaluation Metrics

### Vision Metrics

#### Description Accuracy
Measures how well image descriptions match ground truth using:
- **Word overlap** (Jaccard similarity)
- **BLEU score** (n-gram precision)
- **ROUGE score** (recall-oriented)
- **CLIP similarity** (semantic similarity)

```rust
use llm_test_bench_core::multimodal::evaluation::{VisionEvaluator, VisionMetrics};

let evaluator = VisionEvaluator::new();

let metrics = evaluator.evaluate(
    &request,
    &response,
    Some("A black cat sitting on a laptop keyboard"),
).await?;

println!("Description Accuracy: {:.2}%", metrics.description_accuracy * 100.0);
```

#### Visual Question Answering (VQA) Accuracy
Measures correctness of answers to visual questions:
- **Exact match** (binary)
- **Fuzzy match** (Levenshtein distance)
- **Semantic similarity** (embedding-based)

#### Object Detection F1 Score
For object detection tasks:
- **Precision**: Correct detections / Total detections
- **Recall**: Correct detections / Total objects
- **F1**: Harmonic mean of precision and recall

### Audio Metrics

#### Word Error Rate (WER)
Standard metric for transcription quality:

```
WER = (Substitutions + Deletions + Insertions) / Total Words
```

Lower is better (0.0 = perfect, 1.0 = completely wrong)

```rust
use llm_test_bench_core::multimodal::evaluation::{AudioEvaluator, AudioMetrics};

let evaluator = AudioEvaluator::new();

let metrics = evaluator.evaluate(
    "the quick brown fox",  // Transcription
    "the quick brown fox",  // Reference
).await?;

println!("WER: {:.2}%", metrics.wer * 100.0);
```

#### Character Error Rate (CER)
More granular than WER, measures character-level errors:

```
CER = (Char Substitutions + Deletions + Insertions) / Total Characters
```

#### Audio Quality Scores
For generated audio (TTS):
- **MOS (Mean Opinion Score)**: 1-5 scale
- **Prosody naturalness**: Speech rhythm and intonation
- **Voice similarity**: Match to target voice

### Combined Multi-Modal Metrics

```rust
use llm_test_bench_core::multimodal::evaluation::MultiModalEvaluator;

let evaluator = MultiModalEvaluator::new();

let metrics = evaluator.evaluate(&request, &response, Some(ground_truth)).await?;

println!("Vision Accuracy: {:.2}%",
    metrics.vision.unwrap().description_accuracy * 100.0);
println!("Audio WER: {:.2}%",
    metrics.audio.unwrap().wer * 100.0);
println!("Overall Score: {:.2}%",
    metrics.overall_score * 100.0);
```

---

## Dataset Management

### Creating Multi-Modal Datasets

```rust
use llm_test_bench_core::multimodal::datasets::*;

// Create dataset
let mut dataset = MultiModalDataset::new("vision-qa-v1");

// Add image captioning examples
let example1 = MultiModalExample::vision_captioning(
    "img001",
    "images/cat.jpg",
    "A tabby cat sleeping on a windowsill",
).await?;

dataset.add_example(example1);

// Add VQA examples
let example2 = MultiModalExample::visual_qa(
    "vqa001",
    "images/chart.png",
    "What is the trend shown in this chart?",
    "The chart shows an upward trend from 2020 to 2024",
).await?;

dataset.add_example(example2);

// Add speech transcription examples
let example3 = MultiModalExample::speech_transcription(
    "audio001",
    "audio/sample.mp3",
    "Hello, this is a test of the speech recognition system.",
).await?;

dataset.add_example(example3);

// Calculate task distribution
dataset.calculate_task_distribution();

// Save dataset
dataset.to_json_file("datasets/multimodal-v1.json").await?;
```

### Loading Datasets

```rust
// Load from JSON
let dataset = MultiModalDataset::from_json_file("datasets/multimodal-v1.json").await?;

println!("Dataset: {}", dataset.name);
println!("Examples: {}", dataset.len());

// Filter by task
let caption_examples = dataset.filter_by_task(MultiModalTask::ImageCaptioning);
println!("Captioning examples: {}", caption_examples.len());
```

### Dataset Builder Pattern

```rust
use llm_test_bench_core::multimodal::datasets::DatasetBuilder;

let dataset = DatasetBuilder::new("custom-dataset")
    .with_description("Custom multi-modal evaluation dataset")
    .with_version("1.0.0")
    .add_example(example1)
    .add_example(example2)
    .add_example(example3)
    .build();
```

---

## Best Practices

### 1. Image Preparation

**Optimize Image Size**
```rust
// Resize large images before sending
let image = ImageInput::from_path("large-image.jpg").await?;

// For providers with size limits
if image.size_bytes > 5_000_000 {
    // Resize or compress image
    warn!("Image too large, consider resizing");
}
```

**Choose Appropriate Detail Level**
```rust
let image_part = ImagePart::new(image)
    .with_detail(ImageDetail::High);  // High detail for complex images
    // or ImageDetail::Low for simple images (faster, cheaper)
```

### 2. Audio Preprocessing

**Optimize Audio Format**
```rust
// Prefer compressed formats for large files
// MP3, Opus, AAC are good choices

// For best transcription quality:
// - 16kHz or higher sample rate
// - Mono audio (unless speaker diarization needed)
// - WAV or FLAC for lossless quality
```

**Language Hints**
```rust
let audio_part = AudioPart::new(audio)
    .with_language("en-US");  // Improves accuracy
```

### 3. Cost Optimization

**Vision Costs**
```rust
// Images are priced by token count
// Factors affecting cost:
// - Image size (larger = more tokens)
// - Detail level (high = more tokens)
// - Number of images per request

// Example pricing (OpenAI GPT-4 Vision):
// - Low detail: ~85 tokens per image
// - High detail: ~170-765 tokens depending on size
```

**Audio Costs**
```rust
// Whisper pricing (per minute of audio):
// - $0.006/minute for transcription

// TTS pricing (per character):
// - $0.015/1K characters (standard)
// - $0.030/1K characters (HD voices)
```

### 4. Error Handling

```rust
use llm_test_bench_core::multimodal::*;

async fn robust_vision_request(image_path: &str) -> Result<String> {
    let image = match ImageInput::from_path(image_path).await {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to load image: {}", e);
            return Err(e);
        }
    };

    let request = MultiModalRequest::new("gpt-4-vision-preview")
        .with_text("Describe this image")
        .with_image(image)
        .with_max_tokens(500);

    match provider.complete_multimodal(request).await {
        Ok(response) => Ok(response.text()),
        Err(ProviderError::RateLimitExceeded { retry_after }) => {
            if let Some(delay) = retry_after {
                tokio::time::sleep(delay).await;
                // Retry request
            }
            Err(e.into())
        }
        Err(e) => Err(e.into()),
    }
}
```

### 5. Performance Tips

**Batch Processing**
```rust
// Process multiple images concurrently
use futures::future::join_all;

let image_paths = vec!["img1.jpg", "img2.jpg", "img3.jpg"];

let futures: Vec<_> = image_paths.iter().map(|path| {
    async move {
        let image = ImageInput::from_path(path).await?;
        let request = MultiModalRequest::new("gpt-4-vision-preview")
            .with_text("Describe this image")
            .with_image(image);
        provider.complete_multimodal(request).await
    }
}).collect();

let results = join_all(futures).await;
```

**Streaming for Long Audio**
```rust
// For long audio files, use chunked processing
// Split audio into smaller segments
// Process segments in parallel
// Combine results
```

---

## API Reference

### Core Types

#### `ImageInput`
```rust
pub enum ImageInput {
    Base64 { data: String, media_type: String, metadata: Option<ImageMetadata> },
    Url { url: String, metadata: Option<ImageMetadata> },
    Path { path: PathBuf, metadata: Option<ImageMetadata> },
}

impl ImageInput {
    pub fn from_base64(data: String, format: ImageFormat) -> Self;
    pub fn from_url(url: impl Into<String>) -> Self;
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self>;
    pub async fn read_bytes(&self) -> Result<Vec<u8>>;
    pub async fn to_base64(&self) -> Result<String>;
}
```

#### `AudioInput`
```rust
pub enum AudioInput {
    Base64 { data: String, media_type: String, metadata: Option<AudioMetadata> },
    Url { url: String, metadata: Option<AudioMetadata> },
    Path { path: PathBuf, metadata: Option<AudioMetadata> },
}

impl AudioInput {
    pub fn from_base64(data: String, format: AudioFormat) -> Self;
    pub fn from_url(url: impl Into<String>) -> Self;
    pub async fn from_path(path: impl AsRef<Path>) -> Result<Self>;
    pub async fn read_bytes(&self) -> Result<Vec<u8>>;
    pub async fn to_base64(&self) -> Result<String>;
}
```

#### `MultiModalRequest`
```rust
pub struct MultiModalRequest {
    pub model: String,
    pub content: MultiModalContent,
    pub system: Option<String>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
}

impl MultiModalRequest {
    pub fn new(model: impl Into<String>) -> Self;
    pub fn with_text(self, text: impl Into<String>) -> Self;
    pub fn with_image(self, image: ImageInput) -> Self;
    pub fn with_audio(self, audio: AudioInput) -> Self;
    pub fn with_video(self, video: VideoInput) -> Self;
    pub fn with_system(self, system: impl Into<String>) -> Self;
    pub fn with_max_tokens(self, max_tokens: usize) -> Self;
    pub fn with_temperature(self, temperature: f32) -> Self;
    pub fn with_streaming(self) -> Self;
}
```

#### `MultiModalResponse`
```rust
pub struct MultiModalResponse {
    pub id: String,
    pub model: String,
    pub content: MultiModalContent,
    pub usage: MultiModalUsage,
    pub finish_reason: FinishReason,
    pub created_at: DateTime<Utc>,
}

impl MultiModalResponse {
    pub fn text(&self) -> String;
    pub fn images(&self) -> Vec<ImageOutput>;
}
```

### Evaluation Types

#### `VisionEvaluator`
```rust
pub struct VisionEvaluator;

impl VisionEvaluator {
    pub fn new() -> Self;
    pub async fn evaluate(
        &self,
        request: &MultiModalRequest,
        response: &MultiModalResponse,
        ground_truth: Option<&str>,
    ) -> Result<VisionMetrics>;
}
```

#### `AudioEvaluator`
```rust
pub struct AudioEvaluator;

impl AudioEvaluator {
    pub fn new() -> Self;
    pub async fn evaluate(
        &self,
        transcription: &str,
        reference: &str,
    ) -> Result<AudioMetrics>;
}
```

---

## Roadmap

### Planned Features

**Phase 5.2 - Advanced Vision**
- [ ] Image generation evaluation (DALL-E, Stable Diffusion)
- [ ] Image quality metrics (FID, IS, LPIPS)
- [ ] Fine-grained object detection benchmarks
- [ ] Vision-language model comparison

**Phase 5.3 - Advanced Audio**
- [ ] Speaker diarization evaluation
- [ ] Emotion detection in speech
- [ ] Audio quality assessment (PESQ, POLQA)
- [ ] Music understanding tasks

**Phase 5.4 - Video Enhancement**
- [ ] Action recognition benchmarks
- [ ] Video captioning evaluation
- [ ] Temporal reasoning tasks
- [ ] Multi-frame analysis

**Phase 5.5 - Cross-Modal**
- [ ] Image-text matching
- [ ] Audio-visual synchronization
- [ ] Video-to-text generation
- [ ] Multi-modal retrieval

---

## Support & Resources

### Documentation
- [Main Documentation](../README.md)
- [Provider Guide](PROVIDERS.md)
- [API Reference](https://docs.rs/llm-test-bench-core)

### Examples
- `/examples/vision_qa.rs` - Visual question answering
- `/examples/audio_transcription.rs` - Speech-to-text
- `/examples/multimodal_benchmark.rs` - Full benchmark

### Community
- GitHub Issues: [Report bugs](https://github.com/your-org/llm-test-bench/issues)
- Discussions: [Ask questions](https://github.com/your-org/llm-test-bench/discussions)

---

## Conclusion

The multi-modal evaluation system in LLM Test Bench provides:

✅ **Comprehensive Coverage**: Vision, audio, and video support
✅ **Enterprise Quality**: Production-ready, well-tested code
✅ **Provider Agnostic**: Works with OpenAI, Anthropic, Google, and more
✅ **Flexible Architecture**: Easy to extend with new modalities
✅ **Rich Metrics**: Industry-standard evaluation metrics
✅ **Dataset Management**: Built-in dataset creation and management
✅ **Cost Efficient**: Optimization tips and token tracking

This makes LLM Test Bench the **most comprehensive multi-modal LLM testing framework** available.
