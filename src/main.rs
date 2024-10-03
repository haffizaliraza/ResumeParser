use pdf_extract::extract_text;
use regex::Regex;
use std::fs::File;
use std::io;
use std::path::Path;
use csv::Writer;
use walkdir::WalkDir;

fn extract_info(text: &str) -> (Option<String>, Vec<String>, Vec<String>) {
    // Define regex patterns for identifying sections
    let summary_re = Regex::new(r"(?i)(?:summary|about me|about)\s*([\s\S]+?)(?:\n\s*\n|\z)").unwrap();
    let skills_re = Regex::new(r"(?i)skills\s*([\s\S]+?)(?:\n\s*\n|\z)").unwrap();
    let experience_re = Regex::new(r"(?i)(?:work experience|experience|employment history)\s*([\s\S]+?)(?:\n\s*\n|\z)").unwrap();
    
    // Extract summary from text
    let summary = if let Some(captures) = summary_re.captures(text) {
        Some(captures.get(1).unwrap().as_str().trim().to_string())
    } else {
        None
    };

    // Extract skills
    let mut skills = Vec::new();
    if let Some(captures) = skills_re.captures(text) {
        let skills_text = captures.get(1).unwrap().as_str().trim();
        skills.extend(skills_text.split(',').map(|s| s.trim().to_string()));
    }

    // Extract work experience
    let mut work_experience = Vec::new();
    if let Some(captures) = experience_re.captures(text) {
        let experience_text = captures.get(1).unwrap().as_str().trim();
        work_experience.extend(experience_text.split('\n').map(|s| s.trim().to_string()));
    }

    (summary, skills, work_experience)
}

fn process_pdf(file_path: &Path) -> Option<(String, Option<String>, Vec<String>, Vec<String>)> {
    // Extract text from the PDF file
    if let Ok(text) = extract_text(file_path) {
        // Extract summary, skills, and work experience
        let (summary, skills, work_experience) = extract_info(&text);
        
        // Return if any skills or experience is found
        if !skills.is_empty() || !work_experience.is_empty() {
            return Some((
                file_path.file_name()?.to_string_lossy().into_owned(),
                summary,
                skills,
                work_experience,
            ));
        }
    }
    None
}

fn write_to_csv(file_path: &str, data: Vec<(String, Option<String>, Vec<String>, Vec<String>)>) -> io::Result<()> {
    let file = File::create(file_path)?;
    let mut wtr = Writer::from_writer(file);

    // Write header
    wtr.write_record(&["Filename", "Summary", "Skills", "Work Experience"])?;

    // Write each resume data to the CSV
    for (filename, summary, skills, work_experience) in data {
        let skills_str = skills.join("; ");
        let work_experience_str = work_experience.join("; ");
        wtr.write_record(&[
            filename,
            summary.unwrap_or_else(|| "No summary found".to_string()),
            skills_str,
            work_experience_str,
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let resume_dir = "./resumes";
    let mut results: Vec<(String, Option<String>, Vec<String>, Vec<String>)> = Vec::new();
    let mut file_count = 0; // Counter for processed files

    // Iterate through all PDF files in the directory
    for entry in WalkDir::new(resume_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pdf"))
    {
        let path = entry.path().to_path_buf();
        let filename = path.file_name().unwrap().to_string_lossy().into_owned();
        
        if let Some((filename, summary, skills, work_experience)) = process_pdf(&path) {
            results.push((filename, summary, skills, work_experience));

            // Increment the processed file counter
            file_count += 1;
            println!("Processed file: {}", path.display());
        } else {
            println!("No relevant data found in file: {}", path.display());
        }
    }

    // Write results to CSV file
    let output_csv = "resume_data.csv";
    write_to_csv(output_csv, results)?;

    // Print the number of files processed
    println!("Total files processed: {}", file_count);
    println!("Results have been written to {}", output_csv);

    Ok(())
}
