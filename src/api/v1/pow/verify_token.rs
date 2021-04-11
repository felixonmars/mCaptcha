/*
* Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU Affero General Public License as
* published by the Free Software Foundation, either version 3 of the
* License, or (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU Affero General Public License for more details.
*
* You should have received a copy of the GNU Affero General Public License
* along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use actix_web::{post, web, HttpResponse, Responder};
use m_captcha::cache::messages::VerifyCaptchaResult;
use serde::{Deserialize, Serialize};

use crate::errors::*;
use crate::Data;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaptchaValidateResp {
    pub valid: bool,
}

// API keys are mcaptcha actor names

#[post("/api/v1/siteverify")]
pub async fn validate_captcha_token(
    payload: web::Json<VerifyCaptchaResult>,
    data: web::Data<Data>,
) -> ServiceResult<impl Responder> {
    let res = data
        .captcha
        .validate_verification_tokens(payload.into_inner())
        .await?;
    let payload = CaptchaValidateResp { valid: res };
    println!("{:?}", &payload);
    Ok(HttpResponse::Ok().json(payload))
}

#[cfg(test)]
mod tests {
    use actix_web::http::{header, StatusCode};
    use actix_web::test;
    use m_captcha::pow::PoWConfig;
    use m_captcha::pow::Work;

    use super::*;
    use crate::api::v1::pow::get_config::GetConfigPayload;
    use crate::api::v1::pow::verify_pow::ValidationToken;
    use crate::api::v1::services as v1_services;
    use crate::tests::*;
    use crate::*;

    #[actix_rt::test]
    async fn validate_captcha_token_works() {
        const NAME: &str = "enterprisetken";
        const PASSWORD: &str = "testingpas";
        const EMAIL: &str = "verifyuser@enter.com";
        const VERIFY_CAPTCHA_URL: &str = "/api/v1/mcaptcha/pow/verify";
        const GET_URL: &str = "/api/v1/mcaptcha/pow/config";
        const VERIFY_TOKEN_URL: &str = "/api/v1/siteverify";
        //        const UPDATE_URL: &str = "/api/v1/mcaptcha/domain/token/duration/update";

        {
            let data = Data::new().await;
            delete_user(NAME, &data).await;
        }

        register_and_signin(NAME, EMAIL, PASSWORD).await;
        let (data, _, _signin_resp, token_key) = add_levels_util(NAME, PASSWORD).await;
        let mut app = get_app!(data).await;

        let get_config_payload = GetConfigPayload {
            key: token_key.key.clone(),
        };

        // update and check changes

        let get_config_resp = test::call_service(
            &mut app,
            post_request!(&get_config_payload, GET_URL).to_request(),
        )
        .await;
        assert_eq!(get_config_resp.status(), StatusCode::OK);
        let config: PoWConfig = test::read_body_json(get_config_resp).await;

        let pow = pow_sha256::ConfigBuilder::default()
            .salt(config.salt)
            .build()
            .unwrap();
        let work = pow
            .prove_work(&config.string.clone(), config.difficulty_factor)
            .unwrap();

        let work = Work {
            string: config.string.clone(),
            result: work.result,
            nonce: work.nonce,
            key: token_key.key.clone(),
        };

        let pow_verify_resp = test::call_service(
            &mut app,
            post_request!(&work, VERIFY_CAPTCHA_URL).to_request(),
        )
        .await;
        assert_eq!(pow_verify_resp.status(), StatusCode::OK);
        let client_token: ValidationToken = test::read_body_json(pow_verify_resp).await;

        let validate_payload = VerifyCaptchaResult {
            token: client_token.token.clone(),
            key: token_key.key.clone(),
        };

        let validate_client_token = test::call_service(
            &mut app,
            post_request!(&validate_payload, VERIFY_TOKEN_URL).to_request(),
        )
        .await;
        assert_eq!(validate_client_token.status(), StatusCode::OK);
        let resp: CaptchaValidateResp = test::read_body_json(validate_client_token).await;
        assert!(resp.valid);

        // string not found
        let string_not_found = test::call_service(
            &mut app,
            post_request!(&validate_payload, VERIFY_TOKEN_URL).to_request(),
        )
        .await;
        let resp: CaptchaValidateResp = test::read_body_json(string_not_found).await;
        assert!(!resp.valid);

        let validate_payload = VerifyCaptchaResult {
            token: client_token.token.clone(),
            key: client_token.token.clone(),
        };

        // key not found
        let key_not_found = test::call_service(
            &mut app,
            post_request!(&validate_payload, VERIFY_TOKEN_URL).to_request(),
        )
        .await;
        let resp: CaptchaValidateResp = test::read_body_json(key_not_found).await;
        assert!(!resp.valid);
    }
}