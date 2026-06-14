use captcha_rs::CaptchaBuilder;

pub struct CaptchaService;

impl CaptchaService {
    pub fn generate() -> (String, String, String) {
        let captcha = CaptchaBuilder::new()
            .length(5)
            .width(160)
            .height(50)
            .dark_mode(true)
            .complexity(3)
            .build();

        let answer = captcha.text.to_lowercase();
        let base64 = captcha.to_base64();
        let token = uuid::Uuid::new_v4().to_string();

        (token, answer, base64)
    }
}
