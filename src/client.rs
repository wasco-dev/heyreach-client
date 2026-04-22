use crate::exports::wasco_dev::heyreach_api::heyreach_api::*;
use crate::http::{make_request, make_request_empty, HttpMethod};
use crate::models::*;

// -------- Helper functions for conversion --------

fn map_campaign_status(status: &str) -> CampaignStatus {
    match status.to_lowercase().as_str() {
        "draft" => CampaignStatus::Draft,
        "active" => CampaignStatus::Active,
        "paused" => CampaignStatus::Paused,
        "finished" => CampaignStatus::Finished,
        "canceled" => CampaignStatus::Canceled,
        _ => CampaignStatus::Unknown,
    }
}

fn campaign_status_to_string(status: &CampaignStatus) -> String {
    match status {
        CampaignStatus::Draft => "draft",
        CampaignStatus::Active => "active",
        CampaignStatus::Paused => "paused",
        CampaignStatus::Finished => "finished",
        CampaignStatus::Canceled => "canceled",
        CampaignStatus::Unknown => "unknown",
    }
    .to_string()
}

fn map_list_type(list_type: &str) -> ListType {
    match list_type.to_lowercase().as_str() {
        "leads" => ListType::Leads,
        "companies" => ListType::Companies,
        _ => ListType::Unknown,
    }
}

fn map_webhook_event_type(event_type: &str) -> WebhookEventType {
    match event_type.to_lowercase().as_str() {
        "connectionrequestsent" | "connection_request_sent" | "connection-request-sent" => {
            WebhookEventType::ConnectionRequestSent
        }
        "connectionaccepted" | "connection_accepted" | "connection-accepted" => {
            WebhookEventType::ConnectionAccepted
        }
        "messagesent" | "message_sent" | "message-sent" => WebhookEventType::MessageSent,
        "messagereplied" | "message_replied" | "message-replied" => {
            WebhookEventType::MessageReplied
        }
        _ => WebhookEventType::Unknown,
    }
}

fn webhook_event_type_to_string(event_type: &WebhookEventType) -> String {
    match event_type {
        WebhookEventType::ConnectionRequestSent => "ConnectionRequestSent",
        WebhookEventType::ConnectionAccepted => "ConnectionAccepted",
        WebhookEventType::MessageSent => "MessageSent",
        WebhookEventType::MessageReplied => "MessageReplied",
        WebhookEventType::Unknown => "Unknown",
    }
    .to_string()
}

fn convert_progress_stats(dto: ProgressStatsDto) -> ProgressStats {
    ProgressStats {
        total_users: dto.total_users,
        total_users_in_progress: dto.total_users_in_progress,
        total_users_pending: dto.total_users_pending,
        total_users_finished: dto.total_users_finished,
        total_users_failed: dto.total_users_failed,
        total_users_manually_stopped: dto.total_users_manually_stopped,
        total_users_excluded: dto.total_users_excluded,
    }
}

fn convert_campaign_summary(dto: CampaignSummaryDto) -> CampaignSummary {
    CampaignSummary {
        id: dto.id,
        name: dto.name,
        creation_time: dto.creation_time,
        linkedin_user_list_name: dto.linkedin_user_list_name,
        linkedin_user_list_id: dto.linkedin_user_list_id,
        campaign_account_ids: dto.campaign_account_ids,
        status: map_campaign_status(&dto.status),
        progress_stats: dto.progress_stats.map(convert_progress_stats),
        exclude_already_messaged_global: dto.exclude_already_messaged_global,
        exclude_already_messaged_campaign_accounts: dto.exclude_already_messaged_campaign_accounts,
        exclude_first_connection_campaign_accounts: dto.exclude_first_connection_campaign_accounts,
        exclude_first_connection_global: dto.exclude_first_connection_global,
        exclude_no_profile_picture: dto.exclude_no_profile_picture,
        exclude_list_id: dto.exclude_list_id,
        exclude_in_other_campaigns: dto.exclude_in_other_campaigns,
        exclude_has_other_acc_conversations: dto.exclude_has_other_acc_conversations,
        exclude_contacted_from_sender_in_other_campaign: dto
            .exclude_contacted_from_sender_in_other_campaign,
        organization_unit_id: dto.organization_unit_id,
    }
}

fn convert_lead_dto(dto: LeadDto) -> Lead {
    Lead {
        first_name: dto.first_name,
        last_name: dto.last_name,
        profile_url: dto.profile_url,
        location: dto.location,
        summary: dto.summary,
        company_name: dto.company_name,
        position: dto.position,
        about: dto.about,
        email_address: dto.email_address,
        custom_user_fields: dto
            .custom_user_fields
            .into_iter()
            .map(|f| CustomUserField {
                name: f.name,
                value: f.value,
            })
            .collect(),
    }
}

fn convert_lead(lead: Lead) -> LeadDto {
    LeadDto {
        first_name: lead.first_name,
        last_name: lead.last_name,
        profile_url: lead.profile_url,
        location: lead.location,
        summary: lead.summary,
        company_name: lead.company_name,
        position: lead.position,
        about: lead.about,
        email_address: lead.email_address,
        custom_user_fields: lead
            .custom_user_fields
            .into_iter()
            .map(|f| CustomUserFieldDto {
                name: f.name,
                value: f.value,
            })
            .collect(),
    }
}

// -------- Auth --------

pub fn check_api_key(api_key: &str) -> Result<(), ApiError> {
    make_request_empty(
        HttpMethod::Get,
        "/api/public/auth/CheckApiKey",
        api_key,
        None::<&()>,
    )
}

// -------- Campaigns --------

pub fn campaigns_get_all(api_key: &str, filter: CampaignFilter) -> Result<CampaignPage, ApiError> {
    let filter_dto = CampaignFilterDto {
        offset: filter.offset,
        limit: filter.limit,
        keyword: filter.keyword,
        statuses: filter
            .statuses
            .iter()
            .map(campaign_status_to_string)
            .collect(),
        account_ids: filter.account_ids,
    };

    let response: CampaignPageDto = make_request(
        HttpMethod::Post,
        "/api/public/campaign/GetAll",
        api_key,
        Some(&filter_dto),
    )?;

    // ✅ FIXED: API returns {totalCount, items} directly, NOT {page, items}
    Ok(CampaignPage {
        total_count: response.total_count,
        items: response
            .items
            .into_iter()
            .map(convert_campaign_summary)
            .collect(),
    })
}

pub fn campaigns_get_by_id(api_key: &str, campaign_id: u64) -> Result<CampaignSummary, ApiError> {
    let response: CampaignSummaryDto = make_request(
        HttpMethod::Get,
        &format!("/api/public/campaign/GetById?campaignId={}", campaign_id),
        api_key,
        None::<&()>,
    )?;

    Ok(convert_campaign_summary(response))
}

pub fn campaigns_resume(api_key: &str, campaign_id: u64) -> Result<(), ApiError> {
    make_request_empty(
        HttpMethod::Post,
        &format!("/api/public/campaign/Resume?campaignId={}", campaign_id),
        api_key,
        None::<&()>,
    )
}

pub fn campaigns_pause(api_key: &str, campaign_id: u64) -> Result<(), ApiError> {
    make_request_empty(
        HttpMethod::Post,
        &format!("/api/public/campaign/Pause?campaignId={}", campaign_id),
        api_key,
        None::<&()>,
    )
}

pub fn campaigns_add_leads(
    api_key: &str,
    payload: CampaignAddLeadsRequest,
) -> Result<u32, ApiError> {
    let payload_dto = CampaignAddLeadsRequestDto {
        campaign_id: payload.campaign_id,
        account_lead_pairs: payload
            .account_lead_pairs
            .into_iter()
            .map(|p| AccountLeadPairDto {
                linked_in_account_id: p.linked_in_account_id,
                lead: convert_lead(p.lead),
            })
            .collect(),
    };

    let response: u32 = make_request(
        HttpMethod::Post,
        "/api/public/campaign/AddLeadsToCampaign",
        api_key,
        Some(&payload_dto),
    )?;

    Ok(response)
}

pub fn campaigns_add_leads_v2(
    api_key: &str,
    payload: CampaignAddLeadsRequest,
) -> Result<CampaignAddLeadsV2Result, ApiError> {
    let payload_dto = CampaignAddLeadsRequestDto {
        campaign_id: payload.campaign_id,
        account_lead_pairs: payload
            .account_lead_pairs
            .into_iter()
            .map(|p| AccountLeadPairDto {
                linked_in_account_id: p.linked_in_account_id,
                lead: convert_lead(p.lead),
            })
            .collect(),
    };

    let response: CampaignAddLeadsV2ResultDto = make_request(
        HttpMethod::Post,
        "/api/public/campaign/AddLeadsToCampaignV2",
        api_key,
        Some(&payload_dto),
    )?;

    Ok(CampaignAddLeadsV2Result {
        added_leads_count: response.added_leads_count,
        updated_leads_count: response.updated_leads_count,
        failed_leads_count: response.failed_leads_count,
    })
}

// -------- Lists --------

pub fn lists_get_all(api_key: &str, filter: ListGetAllFilter) -> Result<ListPage, ApiError> {
    let filter_dto = ListGetAllFilterDto {
        offset: filter.offset,
        limit: filter.limit,
        keyword: filter.keyword,
    };

    let response: ListPageDto = make_request(
        HttpMethod::Post,
        "/api/public/list/GetAll",
        api_key,
        Some(&filter_dto),
    )?;

    // ✅ FIXED: API returns {totalCount, items} directly, NOT {page, items}
    Ok(ListPage {
        total_count: response.total_count,
        items: response
            .items
            .into_iter()
            .map(|dto| ListSummary {
                id: dto.id,
                name: dto.name,
                total_items_count: dto.total_items_count,
                list_type: map_list_type(&dto.list_type),
                creation_time: dto.creation_time,
                campaign_ids: dto.campaign_ids,
            })
            .collect(),
    })
}

pub fn lists_get_by_id(api_key: &str, list_id: u64) -> Result<ListSummary, ApiError> {
    let response: ListSummaryDto = make_request(
        HttpMethod::Get,
        &format!("/api/public/list/GetById?listId={}", list_id),
        api_key,
        None::<&()>,
    )?;

    Ok(ListSummary {
        id: response.id,
        name: response.name,
        total_items_count: response.total_items_count,
        list_type: map_list_type(&response.list_type),
        creation_time: response.creation_time,
        campaign_ids: response.campaign_ids,
    })
}

pub fn lists_get_leads(
    api_key: &str,
    list_id: u64,
    offset: u32,
    limit: u32,
    keyword: Option<String>,
) -> Result<ListLeadsPage, ApiError> {
    let request_dto = ListGetLeadsRequestDto {
        list_id,
        offset,
        limit,
        keyword,
    };

    let response: ListLeadsPageDto = make_request(
        HttpMethod::Post,
        "/api/public/list/GetLeadsFromList",
        api_key,
        Some(&request_dto),
    )?;

    // ✅ FIXED: API returns {totalCount, items} directly
    Ok(ListLeadsPage {
        total_count: response.total_count,
        items: response.items.into_iter().map(convert_lead_dto).collect(),
    })
}

pub fn lists_add_leads(api_key: &str, list_id: u64, leads: Vec<Lead>) -> Result<(), ApiError> {
    let request_dto = ListAddLeadsRequestDto {
        list_id,
        leads: leads.into_iter().map(convert_lead).collect(),
    };

    make_request_empty(
        HttpMethod::Post,
        "/api/public/list/AddLeadsToList",
        api_key,
        Some(&request_dto),
    )
}

pub fn lists_add_leads_v2(
    api_key: &str,
    list_id: u64,
    leads: Vec<Lead>,
) -> Result<CampaignAddLeadsV2Result, ApiError> {
    let request_dto = ListAddLeadsRequestDto {
        list_id,
        leads: leads.into_iter().map(convert_lead).collect(),
    };

    let response: CampaignAddLeadsV2ResultDto = make_request(
        HttpMethod::Post,
        "/api/public/list/AddLeadsToListV2",
        api_key,
        Some(&request_dto),
    )?;

    Ok(CampaignAddLeadsV2Result {
        added_leads_count: response.added_leads_count,
        updated_leads_count: response.updated_leads_count,
        failed_leads_count: response.failed_leads_count,
    })
}

pub fn lists_delete_leads(api_key: &str, request: ListLeadDeleteRequest) -> Result<(), ApiError> {
    let request_dto = ListLeadDeleteRequestDto {
        list_id: request.list_id,
        lead_member_ids: request.lead_member_ids,
    };

    make_request_empty(
        HttpMethod::Delete,
        "/api/public/list/DeleteLeadsFromList",
        api_key,
        Some(&request_dto),
    )
}

pub fn lists_delete_leads_by_profile_url(
    api_key: &str,
    request: ListLeadDeleteByProfileUrlRequest,
) -> Result<ListLeadDeleteByProfileUrlResponse, ApiError> {
    let request_dto = ListLeadDeleteByProfileUrlRequestDto {
        list_id: request.list_id,
        profile_urls: request.profile_urls,
    };

    let response: ListLeadDeleteByProfileUrlResponseDto = make_request(
        HttpMethod::Delete,
        "/api/public/list/DeleteLeadsFromListByProfileUrl",
        api_key,
        Some(&request_dto),
    )?;

    Ok(ListLeadDeleteByProfileUrlResponse {
        not_found_in_list: response.not_found_in_list,
    })
}

// -------- Lead & Tags --------

pub fn lead_get(api_key: &str, profile_url: String) -> Result<Lead, ApiError> {
    let request_dto = LeadGetRequestDto { profile_url };

    let response: LeadDto = make_request(
        HttpMethod::Post,
        "/api/public/lead/GetLead",
        api_key,
        Some(&request_dto),
    )?;

    Ok(convert_lead_dto(response))
}

pub fn lead_get_lists(
    api_key: &str,
    request: LeadListsRequest,
) -> Result<LeadListsResponse, ApiError> {
    let request_dto = LeadListsRequestDto {
        email: request.email,
        linkedin_id: request.linkedin_id,
        profile_url: request.profile_url,
        offset: request.offset,
        limit: request.limit,
    };

    let response: LeadListsResponseDto = make_request(
        HttpMethod::Post,
        "/api/public/list/GetListsForLead",
        api_key,
        Some(&request_dto),
    )?;

    // ✅ FIXED: API returns {totalCount, items} directly
    Ok(LeadListsResponse {
        total_count: response.total_count,
        items: response
            .items
            .into_iter()
            .map(|dto| LeadListSummary {
                list_id: dto.list_id,
                list_name: dto.list_name,
            })
            .collect(),
    })
}

pub fn lead_get_tags(api_key: &str, profile_url: String) -> Result<LeadTagsResponse, ApiError> {
    let request_dto = LeadGetRequestDto { profile_url };

    let response: LeadTagsResponseDto = make_request(
        HttpMethod::Post,
        "/api/public/lead/GetTags",
        api_key,
        Some(&request_dto),
    )?;

    Ok(LeadTagsResponse {
        tags: response.tags,
    })
}

pub fn lead_replace_tags(
    api_key: &str,
    request: LeadReplaceTagsRequest,
) -> Result<LeadReplaceTagsResponse, ApiError> {
    let request_dto = LeadReplaceTagsRequestDto {
        lead_profile_url: request.lead_profile_url,
        lead_linked_in_id: request.lead_linked_in_id,
        tags: request.tags,
        create_tag_if_not_existing: request.create_tag_if_not_existing,
    };

    let response: LeadReplaceTagsResponseDto = make_request(
        HttpMethod::Post,
        "/api/public/lead/ReplaceTags",
        api_key,
        Some(&request_dto),
    )?;

    Ok(LeadReplaceTagsResponse {
        new_assigned_tags: response.new_assigned_tags,
    })
}

// -------- Inbox --------

pub fn inbox_get_conversations_v2(
    api_key: &str,
    request: InboxGetConversationsRequest,
) -> Result<InboxConversationPage, ApiError> {
    let request_dto = InboxGetConversationsRequestDto {
        filters: InboxFiltersDto {
            linked_in_account_ids: request.filters.linked_in_account_ids,
            campaign_ids: request.filters.campaign_ids,
            search_string: request.filters.search_string,
            lead_linked_in_id: request.filters.lead_linked_in_id,
            lead_profile_url: request.filters.lead_profile_url,
            seen: request.filters.seen,
        },
        offset: request.offset,
        limit: request.limit,
    };

    let response: InboxConversationPageDto = make_request(
        HttpMethod::Post,
        "/api/public/inbox/GetConversationsV2",
        api_key,
        Some(&request_dto),
    )?;

    // ✅ FIXED: API returns {totalCount, items} directly
    Ok(InboxConversationPage {
        total_count: response.total_count,
        items: response
            .items
            .into_iter()
            .map(|dto| InboxConversationSummary {
                conversation_id: dto.conversation_id,
                linked_in_account_id: dto.linked_in_account_id,
                lead_profile_url: dto.lead_profile_url,
                last_message_snippet: dto.last_message_snippet,
                seen: dto.seen,
            })
            .collect(),
    })
}

pub fn inbox_send_message(api_key: &str, request: InboxSendMessageRequest) -> Result<(), ApiError> {
    let request_dto = InboxSendMessageRequestDto {
        message: request.message,
        subject: request.subject,
        conversation_id: request.conversation_id,
        linked_in_account_id: request.linked_in_account_id,
    };

    make_request_empty(
        HttpMethod::Post,
        "/api/public/inbox/SendMessage",
        api_key,
        Some(&request_dto),
    )
}

// -------- LinkedIn Accounts --------

pub fn li_account_get_all(
    api_key: &str,
    filter: LiAccountFilter,
) -> Result<LiAccountPage, ApiError> {
    let filter_dto = LiAccountFilterDto {
        offset: filter.offset,
        limit: filter.limit,
        keyword: filter.keyword,
    };

    let response: LiAccountPageDto = make_request(
        HttpMethod::Post,
        "/api/public/li_account/GetAll",
        api_key,
        Some(&filter_dto),
    )?;

    Ok(LiAccountPage {
        total_count: response.total_count,
        items: response
            .items
            .into_iter()
            .map(|dto| LiAccountSummary {
                id: dto.id,
                email_address: dto.email_address,
                first_name: dto.first_name,
                last_name: dto.last_name,
                is_active: dto.is_active,
                active_campaigns: dto.active_campaigns,
                auth_is_valid: dto.auth_is_valid,
                is_valid_navigator: dto.is_valid_navigator,
                is_valid_recruiter: dto.is_valid_recruiter,
            })
            .collect(),
    })
}

// -------- Webhooks --------

pub fn webhooks_create(api_key: &str, request: CreateWebhookRequest) -> Result<Webhook, ApiError> {
    let request_dto = CreateWebhookRequestDto {
        webhook_name: request.webhook_name,
        webhook_url: request.webhook_url,
        event_type: webhook_event_type_to_string(&request.event_type),
        campaign_ids: request.campaign_ids,
        is_active: request.is_active,
    };

    let response: WebhookDto = make_request(
        HttpMethod::Post,
        "/api/public/webhooks/CreateWebhook",
        api_key,
        Some(&request_dto),
    )?;

    Ok(Webhook {
        id: response.id,
        webhook_name: response.webhook_name,
        webhook_url: response.webhook_url,
        event_type: map_webhook_event_type(&response.event_type),
        campaign_ids: response.campaign_ids,
        is_active: response.is_active,
    })
}

pub fn webhooks_get_by_id(api_key: &str, webhook_id: u64) -> Result<Webhook, ApiError> {
    let response: WebhookDto = make_request(
        HttpMethod::Get,
        &format!(
            "/api/public/webhooks/GetWebhookById?webhookId={}",
            webhook_id
        ),
        api_key,
        None::<&()>,
    )?;

    Ok(Webhook {
        id: response.id,
        webhook_name: response.webhook_name,
        webhook_url: response.webhook_url,
        event_type: map_webhook_event_type(&response.event_type),
        campaign_ids: response.campaign_ids,
        is_active: response.is_active,
    })
}

pub fn webhooks_get_all(api_key: &str, filter: GetWebhooksFilter) -> Result<WebhookPage, ApiError> {
    let filter_dto = GetWebhooksFilterDto {
        offset: filter.offset,
        limit: filter.limit,
    };

    let response: WebhookPageDto = make_request(
        HttpMethod::Post,
        "/api/public/webhooks/GetAllWebhooks",
        api_key,
        Some(&filter_dto),
    )?;

    // ✅ FIXED: API returns {totalCount, items} directly
    Ok(WebhookPage {
        total_count: response.total_count,
        items: response
            .items
            .into_iter()
            .map(|dto| Webhook {
                id: dto.id,
                webhook_name: dto.webhook_name,
                webhook_url: dto.webhook_url,
                event_type: map_webhook_event_type(&dto.event_type),
                campaign_ids: dto.campaign_ids,
                is_active: dto.is_active,
            })
            .collect(),
    })
}

pub fn webhooks_delete(api_key: &str, webhook_id: u64) -> Result<(), ApiError> {
    make_request_empty(
        HttpMethod::Delete,
        &format!(
            "/api/public/webhooks/DeleteWebhook?webhookId={}",
            webhook_id
        ),
        api_key,
        None::<&()>,
    )
}
